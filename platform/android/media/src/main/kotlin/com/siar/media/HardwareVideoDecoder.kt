package com.siar.media

import android.media.MediaCodec
import android.media.MediaFormat
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Mirror of [HardwareVideoEncoder] for decode — same synchronous
 * dequeue-loop design, same ByteBuffer-based (not Surface-based) I/O
 * for the same reason: the JNI boundary needs bytes on at least one
 * side, and this is the correctness-first version of that path.
 *
 * One real asymmetry from the encoder: MediaCodec decoders configured
 * without an output `Surface` produce frames in a
 * `MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible`-family
 * layout, and Android does not guarantee a single fixed stride/layout
 * across vendors for that format the way an encoder's *input*
 * (which this app fully controls) does. `extractYuv420` below reads
 * the actual per-plane strides MediaCodec reports rather than
 * assuming tightly-packed planes — the same "don't trust a fixed
 * layout, read the real stride" principle `siar-media-av1`'s dav1d
 * decoder applies to `Dav1dPicture`'s planes.
 */
internal class HardwareVideoDecoder(
    private val handle: Long,
    private val codec: CodecId,
    mimeType: String,
    width: Int,
    height: Int,
) {
    private val mediaCodec: MediaCodec = MediaCodec.createDecoderByType(mimeType)
    private val running = AtomicBoolean(false)
    private var workerThread: Thread? = null
    private val expectedWidth = width
    private val expectedHeight = height

    init {
        val format = MediaFormat.createVideoFormat(mimeType, width, height)
        // No output Surface (`null`) — decoded pictures come back as
        // buffers we read directly, per this class's doc comment.
        mediaCodec.configure(format, null, null, 0)
    }

    fun start() {
        if (running.getAndSet(true)) return
        mediaCodec.start()
        val thread = Thread(::runLoop, "siar-decoder-${codec.name.lowercase()}")
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
            NativeMediaBridge.onCodecError(handle, e.message ?: e.toString())
            running.set(false)
        }
    }

    private fun feedInputIfAvailable() {
        val inputIndex = mediaCodec.dequeueInputBuffer(DEQUEUE_TIMEOUT_US)
        if (inputIndex < 0) return

        val packet = NativeMediaBridge.nextEncodedFrame(handle) ?: return
        val presentationTimeUs = NativeMediaBridge.nextEncodedFrameTimestampUs(handle)

        val inputBuffer: ByteBuffer = mediaCodec.getInputBuffer(inputIndex)
            ?: throw IllegalStateException("getInputBuffer($inputIndex) returned null after a valid dequeueInputBuffer")
        inputBuffer.clear()
        inputBuffer.put(packet)
        mediaCodec.queueInputBuffer(inputIndex, 0, packet.size, presentationTimeUs, 0)
    }

    private fun drainOutput(bufferInfo: MediaCodec.BufferInfo) {
        val outputIndex = mediaCodec.dequeueOutputBuffer(bufferInfo, DEQUEUE_TIMEOUT_US)
        if (outputIndex < 0) return // TRY_AGAIN_LATER / FORMAT_CHANGED / (deprecated) BUFFERS_CHANGED — nothing to do

        val format = mediaCodec.getOutputFormat(outputIndex)
        val outputBuffer: ByteBuffer = mediaCodec.getOutputBuffer(outputIndex)
            ?: throw IllegalStateException("getOutputBuffer($outputIndex) returned null after a valid dequeueOutputBuffer")

        val result = extractYuv420(outputBuffer, format, bufferInfo)
        NativeMediaBridge.onDecodedFrame(handle, result.y, result.u, result.v, result.width, result.height, bufferInfo.presentationTimeUs)

        mediaCodec.releaseOutputBuffer(outputIndex, false)
    }

    /**
     * Reads MediaCodec's actual reported stride/slice-height (falling
     * back to the configured width/height if the format doesn't carry
     * them, which some vendor implementations omit) and copies each
     * plane row by row — never a single bulk copy of `stride * height`
     * bytes, which would pull padding into what's supposed to be
     * tightly-packed output. Same reasoning `siar-media-av1::decoder`
     * applies to `Dav1dPicture`, applied here to `MediaFormat`'s
     * `KEY_STRIDE`/`KEY_SLICE_HEIGHT`.
     *
     * Assumes planar (not semi-planar/NV12-style interleaved U+V)
     * output — `COLOR_FormatYUV420Flexible` from a decoder without a
     * Surface is planar-or-semi-planar depending on vendor; a
     * production build needs to branch on the actual reported
     * `COLOR_FormatYUV420...` constant and handle both. That branch
     * isn't included here — flagging it explicitly rather than
     * silently mishandling semi-planar output as if it were planar,
     * which would produce corrupted chroma on any device that reports
     * NV12-style output.
     */
    private data class Yuv420Result(val y: ByteArray, val u: ByteArray, val v: ByteArray, val width: Int, val height: Int)

    private fun extractYuv420(buffer: ByteBuffer, format: MediaFormat, info: MediaCodec.BufferInfo): Yuv420Result {
        val width = if (format.containsKey(MediaFormat.KEY_WIDTH)) format.getInteger(MediaFormat.KEY_WIDTH) else expectedWidth
        val height = if (format.containsKey(MediaFormat.KEY_HEIGHT)) format.getInteger(MediaFormat.KEY_HEIGHT) else expectedHeight
        val yStride = if (format.containsKey(MediaFormat.KEY_STRIDE)) format.getInteger(MediaFormat.KEY_STRIDE) else width
        val sliceHeight = if (format.containsKey(MediaFormat.KEY_SLICE_HEIGHT)) format.getInteger(MediaFormat.KEY_SLICE_HEIGHT) else height

        val chromaWidth = (width + 1) / 2
        val chromaHeight = (height + 1) / 2
        val chromaStride = (yStride + 1) / 2

        val yPlane = ByteArray(width * height)
        for (row in 0 until height) {
            buffer.position(info.offset + row * yStride)
            buffer.get(yPlane, row * width, width)
        }

        val uPlaneStart = info.offset + yStride * sliceHeight
        val uPlane = ByteArray(chromaWidth * chromaHeight)
        for (row in 0 until chromaHeight) {
            buffer.position(uPlaneStart + row * chromaStride)
            buffer.get(uPlane, row * chromaWidth, chromaWidth)
        }

        val vPlaneStart = uPlaneStart + chromaStride * (sliceHeight / 2)
        val vPlane = ByteArray(chromaWidth * chromaHeight)
        for (row in 0 until chromaHeight) {
            buffer.position(vPlaneStart + row * chromaStride)
            buffer.get(vPlane, row * chromaWidth, chromaWidth)
        }

        return Yuv420Result(yPlane, uPlane, vPlane, width, height)
    }

    companion object {
        private const val DEQUEUE_TIMEOUT_US = 10_000L
        private const val STOP_JOIN_TIMEOUT_MS = 2_000L
    }
}
