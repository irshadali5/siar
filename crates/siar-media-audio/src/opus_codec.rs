//! Verified against `opus 0.3.1`'s actual source: `Encoder::new(rate,
//! channels, Application)`, `encoder.encode_vec(&[i16], max_size) ->
//! Result<Vec<u8>>`, `Decoder::new(rate, channels)`,
//! `decoder.decode(&[u8], &mut [i16], fec) -> Result<usize>` (samples
//! *per channel*, per that method's own doc comment — `decode_frame`
//! below multiplies back out by channel count before returning). The
//! crate's own `unsafe` is confined to `audiopus_sys`'s FFI bindings to
//! libopus; nothing in this file is `unsafe` itself.

use opus::{
    Application, Bitrate, Channels, Decoder as OpusDecoderInner, Encoder as OpusEncoderInner,
};
use siar_media_core::{AudioDecoder, AudioEncoder, DecodeError, EncodeError, EncodedAudioFrame};

/// Opus's own maximum packet size recommendation (from its docs: 4000
/// bytes is enough headroom for any valid encoder configuration this
/// crate uses). Fixed rather than computed because `encode_vec`
/// allocates exactly `max_size` up front — this is deliberately
/// generous, not a tight bound.
const MAX_PACKET_BYTES: usize = 4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannels {
    Mono,
    Stereo,
}

impl From<AudioChannels> for Channels {
    fn from(c: AudioChannels) -> Self {
        match c {
            AudioChannels::Mono => Channels::Mono,
            AudioChannels::Stereo => Channels::Stereo,
        }
    }
}

impl AudioChannels {
    fn count(self) -> usize {
        match self {
            AudioChannels::Mono => 1,
            AudioChannels::Stereo => 2,
        }
    }
}

pub struct OpusEncoder {
    inner: OpusEncoderInner,
    channels: AudioChannels,
}

impl OpusEncoder {
    /// `sample_rate` must be one of Opus's fixed set (8000/12000/16000/
    /// 24000/48000 Hz) — an unsupported rate surfaces as
    /// `EncodeError::Backend` from libopus itself rather than being
    /// validated here, since libopus's own error message is more
    /// precise than anything this wrapper could reconstruct.
    /// `target_bitrate_bps` matches architecture doc §10's "keep audio
    /// simple: Opus everywhere" — `Application::Voip` (this
    /// constructor's fixed choice) is Opus's tuning for speech
    /// intelligibility over raw fidelity, which is the right tradeoff
    /// for a messenger's voice calls.
    pub fn new(
        sample_rate: u32,
        channels: AudioChannels,
        target_bitrate_bps: i32,
    ) -> Result<Self, EncodeError> {
        let mut inner = OpusEncoderInner::new(sample_rate, channels.into(), Application::Voip)
            .map_err(|e| EncodeError::Backend(format!("opus_encoder_create: {e}")))?;
        inner
            .set_bitrate(Bitrate::Bits(target_bitrate_bps))
            .map_err(|e| EncodeError::Backend(format!("opus set_bitrate: {e}")))?;
        Ok(Self { inner, channels })
    }
}

impl AudioEncoder for OpusEncoder {
    fn encode(&mut self, pcm: &[i16]) -> Result<EncodedAudioFrame, EncodeError> {
        if pcm.is_empty() || !pcm.len().is_multiple_of(self.channels.count()) {
            return Err(EncodeError::Unsupported(format!(
                "pcm buffer length {} is not a multiple of {} channel(s)",
                pcm.len(),
                self.channels.count()
            )));
        }
        let data = self
            .inner
            .encode_vec(pcm, MAX_PACKET_BYTES)
            .map_err(|e| EncodeError::Backend(format!("opus_encode: {e}")))?;
        Ok(EncodedAudioFrame {
            data,
            timestamp_micros: 0,
        })
    }
}

pub struct OpusDecoder {
    inner: OpusDecoderInner,
    channels: AudioChannels,
    /// Opus frames are at most 120ms; at 48kHz stereo that's 11520
    /// samples. Fixed scratch buffer sized for the worst case across
    /// every rate/channel combination this wrapper supports, so
    /// `decode` never needs to guess a buffer size per call.
    scratch: Vec<i16>,
}

impl OpusDecoder {
    pub fn new(sample_rate: u32, channels: AudioChannels) -> Result<Self, DecodeError> {
        let inner = OpusDecoderInner::new(sample_rate, channels.into())
            .map_err(|e| DecodeError::Backend(format!("opus_decoder_create: {e}")))?;
        const MAX_SAMPLES_PER_CHANNEL_120MS_AT_48KHZ: usize = 5760;
        let scratch = vec![0i16; MAX_SAMPLES_PER_CHANNEL_120MS_AT_48KHZ * channels.count()];
        Ok(Self {
            inner,
            channels,
            scratch,
        })
    }
}

impl AudioDecoder for OpusDecoder {
    fn decode(&mut self, packet: &[u8], fec: bool) -> Result<Vec<i16>, DecodeError> {
        // Opus's own convention (per `Decoder::decode`'s doc comment):
        // an empty slice means "this packet was lost," which is exactly
        // architecture doc §46's "handle codec... fails" pattern applied
        // to a single dropped audio packet rather than a whole
        // hardware encoder — packet loss is normal on real networks,
        // not an error condition this wrapper should reject.
        let samples_per_channel = self
            .inner
            .decode(packet, &mut self.scratch, fec)
            .map_err(|e| DecodeError::Backend(format!("opus_decode: {e}")))?;
        let total_samples = samples_per_channel * self.channels.count();
        Ok(self.scratch[..total_samples].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real Opus round trip: encode a sine-ish tone, decode it back,
    /// and confirm the decoder produces the same sample count Opus's
    /// own framing implies (20ms @ 48kHz mono = 960 samples). This
    /// exercises the actual libopus library through the FFI boundary —
    /// it's not a mock.
    #[test]
    fn encode_then_decode_round_trips_sample_count() {
        let mut encoder = OpusEncoder::new(48_000, AudioChannels::Mono, 32_000).unwrap();
        let mut decoder = OpusDecoder::new(48_000, AudioChannels::Mono).unwrap();

        let frame_len = 960; // 20ms at 48kHz
        let pcm: Vec<i16> = (0..frame_len)
            .map(|i| ((i as f32 * 0.05).sin() * 5000.0) as i16)
            .collect();

        let encoded = encoder.encode(&pcm).unwrap();
        assert!(!encoded.data.is_empty());

        let decoded = decoder.decode(&encoded.data, false).unwrap();
        assert_eq!(decoded.len(), frame_len);
    }

    #[test]
    fn decoding_a_lost_packet_uses_plc_and_still_returns_a_frame() {
        let mut decoder = OpusDecoder::new(48_000, AudioChannels::Mono).unwrap();
        // First, prime the decoder with a real packet so it has state
        // to conceal from — decoding loss with no prior packet is a
        // degenerate case libopus itself doesn't promise much about.
        let mut encoder = OpusEncoder::new(48_000, AudioChannels::Mono, 32_000).unwrap();
        let pcm = vec![0i16; 960];
        let encoded = encoder.encode(&pcm).unwrap();
        decoder.decode(&encoded.data, false).unwrap();

        // Empty slice = lost packet (this crate's convention, matching
        // opus's own `decode` doc comment).
        let concealed = decoder.decode(&[], false).unwrap();
        assert!(!concealed.is_empty());
    }

    #[test]
    fn rejects_pcm_length_not_matching_channel_count() {
        let mut encoder = OpusEncoder::new(48_000, AudioChannels::Stereo, 32_000).unwrap();
        let odd_length_pcm = vec![0i16; 3]; // not a multiple of 2 channels
        assert!(matches!(
            encoder.encode(&odd_length_pcm),
            Err(EncodeError::Unsupported(_))
        ));
    }
}
