package com.siar.media

import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Build
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Real `android.media.MediaCodec` hardware (or software-fallback,
 * device-dependent — see architecture doc §4's negotiation, which
 * decides *which* codec/implementation to request; this class just
 * drives whichever `mimeType` it's given) video encoder.
 *
 * Uses the synchronous dequeue-buffer API rather than the async
 * callback API — simpler to reason about correctly without a device to
 * test timing edge cases against, at the cost of one dedicated polling
 * thread per encoder instance. Also ByteBuffer-based input, not
 * Surface-based — architecture doc §9 prefers a Surface-fed pipeline to
 * avoid the extra copy; this is the correctness-first baseline that
 * gets frame bytes from Rust (over the JNI boundary, which fundamentally
 * needs a byte array on one side or the other) into the encoder. A
 * Surface-based path is real future work, not a correctness gap in
 * what's here.
 */
internal class HardwareVideoEncoder(
    private val handle: Long,
    private val codec: CodecId,
    mimeType: String,
    width: Int,
    height: Int,
    bitrateBps: Int,
    frameRate: Int,
    keyFrameIntervalSeconds: Int,
) {
    private val mediaCodec: MediaCodec = MediaCodec.createEncoderByType(mimeType)
    private val running = AtomicBoolean(false)
    private var workerThread: Thread? = null

    init {
        val format = MediaFormat.createVideoFormat(mimeType, width, height).apply {
            setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible)
            setInteger(MediaFormat.KEY_BIT_RATE, bitrateBps)
            setInteger(MediaFormat.KEY_FRAME_RATE, frameRate)
            setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, keyFrameIntervalSeconds)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                // Constant-bitrate mode fits a realtime call better than
                // MediaCodec's default (which varies by codec/vendor) —
                // matches architecture doc §51's "explicit session
                // configuration" once a bitrate is negotiated, rather
                // than leaving rate control to whatever a given vendor
                // driver defaults to.
                setInteger(MediaFormat.KEY_BITRATE_MODE, MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)
            }
        }
        mediaCodec.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
    }

    fun start() {
        if (running.getAndSet(true)) return
        mediaCodec.start()
        val thread = Thread(::runLoop, "siar-encoder-${codec.name.lowercase()}")
        workerThread = thread
        thread.start()
    }

    fun stop() {
        if (!running.getAndSet(false)) return
        workerThread?.join(STOP_JOIN_TIMEOUT_MS)
        workerThread = null
        try {
            mediaCodec.stop()
        } finally {
            mediaCodec.release()
        }
    }

    private fun runLoop() {
        val bufferInfo = MediaCodec.BufferInfo()
        try {
            while (running.get()) {
                feedInputIfAvailable()
                drainOutput(bufferInfo)
            }
        } catch (e: Exception) {
            // Architecture doc §45: a codec failure mid-call should be
            // reported for renegotiation, not silently swallowed or
            // left to crash the process. Rust decides what "try the
            // next codec in the fallback chain" means; this thread's
            // only job is to say clearly that this encoder instance is
            // no longer usable.
            NativeMediaBridge.onCodecError(handle, e.message ?: e.toString())
            running.set(false)
        }
    }

    private fun feedInputIfAvailable() {
        val inputIndex = mediaCodec.dequeueInputBuffer(DEQUEUE_TIMEOUT_US)
        if (inputIndex < 0) return // no input buffer free right now — normal, not an error

        val frame = NativeMediaBridge.nextRawFrame(handle)
        if (frame == null) {
            // Nothing queued from Rust yet. Returning the buffer
            // unused (queueing zero bytes, no EOS) would waste a
            // dequeue/queue round trip on most vendor implementations
            // for no benefit — the buffer just stays dequeued and
            // available for the next loop iteration in practice on
            // every MediaCodec implementation this targets, since we
            // never called queueInputBuffer for it. Simply not queueing
            // is the correct "nothing to do yet" here.
            return
        }
        val presentationTimeUs = NativeMediaBridge.nextRawFrameTimestampUs(handle)

        val inputBuffer: ByteBuffer = mediaCodec.getInputBuffer(inputIndex)
            ?: throw IllegalStateException("getInputBuffer($inputIndex) returned null after a valid dequeueInputBuffer")
        inputBuffer.clear()
        inputBuffer.put(frame)
        mediaCodec.queueInputBuffer(inputIndex, 0, frame.size, presentationTimeUs, 0)
    }

    private fun drainOutput(bufferInfo: MediaCodec.BufferInfo) {
        val outputIndex = mediaCodec.dequeueOutputBuffer(bufferInfo, DEQUEUE_TIMEOUT_US)
        when {
            outputIndex >= 0 -> {
                val outputBuffer: ByteBuffer = mediaCodec.getOutputBuffer(outputIndex)
                    ?: throw IllegalStateException("getOutputBuffer($outputIndex) returned null after a valid dequeueOutputBuffer")

                // MediaCodec's codec-config buffer (SPS/PPS for H.264,
                // VPS/SPS/PPS for H.265) arrives as its own buffer with
                // BUFFER_FLAG_CODEC_CONFIG before real frame data — it
                // must be forwarded too (a decoder needs it to make
                // sense of the frames that follow) but isn't itself a
                // displayable keyframe, hence `isKeyFrame` checks
                // `BUFFER_FLAG_KEY_FRAME` specifically, not just
                // "config flag absent."
                val data = ByteArray(bufferInfo.size)
                outputBuffer.position(bufferInfo.offset)
                outputBuffer.limit(bufferInfo.offset + bufferInfo.size)
                outputBuffer.get(data)

                val isKeyFrame = (bufferInfo.flags and MediaCodec.BUFFER_FLAG_KEY_FRAME) != 0
                NativeMediaBridge.onEncodedFrame(handle, codec.wireValue, data, isKeyFrame, bufferInfo.presentationTimeUs)

                mediaCodec.releaseOutputBuffer(outputIndex, false)
            }
            // INFO_OUTPUT_FORMAT_CHANGED / INFO_TRY_AGAIN_LATER /
            // INFO_OUTPUT_BUFFERS_CHANGED (the last one deprecated
            // since API 21, MediaCodec still returns it on some
            // vendor implementations) all need no action here — none
            // of them carry frame data.
            else -> Unit
        }
    }

    companion object {
        private const val DEQUEUE_TIMEOUT_US = 10_000L // 10ms
        private const val STOP_JOIN_TIMEOUT_MS = 2_000L
    }
}
