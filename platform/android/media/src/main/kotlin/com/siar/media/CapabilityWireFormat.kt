package com.siar.media

import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Codec identifiers shared between this file, [NativeMediaBridge], and
 * Rust's `jni_bridge.rs` — the three places this numbering must agree.
 * `Int` on the Kotlin side, `u8` on the Rust side; values must stay
 * under 256 for that to keep working, which four codecs is nowhere
 * close to.
 */
internal enum class CodecId(val wireValue: Int) {
    AV1(0),
    H264(1),
    H265(2),
}

internal data class CodecCapabilityEntry(
    val codec: CodecId,
    val hardwareAccelerated: Boolean,
    val canEncode: Boolean,
    val canDecode: Boolean,
    val maxWidth: Int,
    val maxHeight: Int,
    val maxFps: Int,
)

/**
 * Fixed binary layout, little-endian throughout — mirrored exactly by
 * `jni_bridge.rs::decode_capabilities`. Deliberately not JSON/protobuf:
 * see [NativeMediaBridge]'s doc comment on why a payload this small
 * doesn't need a general-purpose format.
 *
 * ```
 * u8      entry_count
 * repeated entry_count times:
 *   u8    codec_id        (CodecId.wireValue)
 *   u8    hardware (0/1)
 *   u8    can_encode (0/1)
 *   u8    can_decode (0/1)
 *   u32   max_width        (little-endian)
 *   u32   max_height       (little-endian)
 *   u32   max_fps          (little-endian)
 * ```
 */
internal object CapabilityWireFormat {
    fun encode(entries: List<CodecCapabilityEntry>): ByteArray {
        require(entries.size <= 255) { "wire format's entry_count is a single byte; got ${entries.size} entries" }

        val out = ByteArrayOutputStream()
        out.write(entries.size)

        for (entry in entries) {
            out.write(entry.codec.wireValue)
            out.write(if (entry.hardwareAccelerated) 1 else 0)
            out.write(if (entry.canEncode) 1 else 0)
            out.write(if (entry.canDecode) 1 else 0)
            out.write(u32LeBytes(entry.maxWidth))
            out.write(u32LeBytes(entry.maxHeight))
            out.write(u32LeBytes(entry.maxFps))
        }

        return out.toByteArray()
    }

    private fun u32LeBytes(value: Int): ByteArray =
        ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(value).array()
}
