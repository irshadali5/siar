package com.siar.media

import android.media.MediaCodecInfo
import android.media.MediaCodecList

/**
 * Real `MediaCodecList` enumeration (architecture doc §3: "query
 * MediaCodecList... Google explicitly recommends querying for a codec
 * supporting the requested MediaFormat"). This is the piece I held
 * back on earlier when the plan was "Rust drives JNI calls into
 * Android APIs" — as plain Kotlin against the Android SDK it's
 * ordinary, statically-typed, compiler-checked code, not a
 * stringly-typed JNI call site. `NativeMediaBridge.reportCapabilities`
 * is the only place this touches Rust, and that's Kotlin calling
 * Rust, not the other way around.
 */
internal object MediaCodecCapabilities {
    private val MIME_TYPES = mapOf(
        CodecId.AV1 to "video/av01",
        CodecId.H264 to "video/avc",
        CodecId.H265 to "video/hevc",
    )

    /**
     * Probes every codec in [MIME_TYPES] for both encode and decode
     * support using `MediaCodecList.REGULAR_CODECS` (hardware-preferred
     * ordering — architecture doc §4's encode priority already puts
     * hardware ahead of software on the Rust side, so this doesn't
     * need to pre-filter to hardware-only; it reports what the device
     * has and lets `siar-media-core::negotiation` apply policy).
     */
    fun probe(): List<CodecCapabilityEntry> {
        val codecList = MediaCodecList(MediaCodecList.REGULAR_CODECS)
        val entries = mutableListOf<CodecCapabilityEntry>()

        for ((codecId, mimeType) in MIME_TYPES) {
            val encodeInfo = findCapableCodec(codecList, mimeType, isEncoder = true)
            val decodeInfo = findCapableCodec(codecList, mimeType, isEncoder = false)

            if (encodeInfo == null && decodeInfo == null) continue

            // Prefer the encoder's reported limits when both exist —
            // encode is almost always the tighter constraint (encoders
            // commonly support a narrower resolution/profile range than
            // decoders on the same hardware block).
            val reference = encodeInfo ?: decodeInfo!!
            val videoCaps = reference.getCapabilitiesForType(mimeType).videoCapabilities

            entries += CodecCapabilityEntry(
                codec = codecId,
                hardwareAccelerated = isHardwareAccelerated(reference),
                canEncode = encodeInfo != null,
                canDecode = decodeInfo != null,
                maxWidth = videoCaps?.supportedWidths?.upper ?: 0,
                maxHeight = videoCaps?.supportedHeights?.upper ?: 0,
                maxFps = videoCaps?.supportedFrameRates?.upper?.toInt() ?: 0,
            )
        }

        return entries
    }

    private fun findCapableCodec(
        codecList: MediaCodecList,
        mimeType: String,
        isEncoder: Boolean,
    ): MediaCodecInfo? =
        codecList.codecInfos.firstOrNull { info ->
            info.isEncoder == isEncoder && info.supportedTypes.any { it.equals(mimeType, ignoreCase = true) }
        }

    /**
     * `MediaCodecInfo.isHardwareAccelerated()` is API 29+; this module
     * targets current Android (architecture doc §22-23's arm64-v8a
     * priority implies a recent minSdk), so no reflection fallback for
     * pre-29 devices — if this project's actual minSdk ends up lower
     * than 29, this is the one line that needs revisiting alongside
     * that decision, not something to silently work around here.
     */
    private fun isHardwareAccelerated(info: MediaCodecInfo): Boolean = info.isHardwareAccelerated
}
