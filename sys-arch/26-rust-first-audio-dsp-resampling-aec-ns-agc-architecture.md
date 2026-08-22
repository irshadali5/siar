# Part 26 — Rust-First Audio DSP, Resampling, AEC/NS/AGC & Hardware-Aware Audio Pipeline Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 26 of 27 additional production-hardening parts  
**Primary language:** Rust  
**Primary design philosophy:** Rust-first, but not Rust-at-any-cost  
**Target platforms:** Android first, then Linux, Windows, macOS, iOS where supported  
**Primary goals:** production-quality low-latency audio, Opus-compatible sample-rate handling, pure-Rust DSP wherever practical, hardware/platform acceleration where it materially improves quality or avoids unrealistic low-level implementation work, bounded latency, low battery use, robust AEC/NS/AGC, drift correction, glitch resistance, and clean integration with the existing P2P architecture

---

# 1. Purpose

The communication platform already defines:

```text
Opus audio
video calling
routing
multipath
battery scheduling
resource limits
Android hardware media
daemon/runtime
diagnostics
```

but a production voice/audio path needs more than:

```text
microphone
→ Opus encoder
→ network
→ Opus decoder
→ speaker
```

Real devices introduce:

```text
sample-rate mismatch
clock drift
echo
background noise
microphone gain variation
speaker feedback
packet jitter
packet loss
audio focus changes
Bluetooth routing
wired headset routing
device DSP
thermal/power limits
```

The system therefore needs a dedicated audio-processing architecture.

---

# 2. Rust-First Principle

Use Rust for everything that can be implemented cleanly and maintained safely without dropping into unnecessary low-level platform complexity.

Prefer Rust for:

```text
resampling
channel conversion
mixing
gain staging
limiting
high-pass filters
DC removal
audio framing
jitter buffering
packet-loss concealment coordination
clock drift estimation
sample-count correction
voice activity state
audio-level metering
silence suppression policy
DSP graph orchestration
latency accounting
buffer pools
diagnostics
test harnesses
```

Use platform/native capabilities when:

```text
hardware DSP is materially better
OS already provides optimized acoustic processing
implementation requires deep vendor/audio-driver integration
a mature production AEC/NS implementation is not reasonably replaceable in pure Rust
```

The rule is:

> **Keep policy, orchestration, buffering, timing, and portable DSP in Rust. Hide unavoidable platform DSP behind small Rust traits.**

---

# 3. Do Not Reimplement Difficult DSP for Ideology

AEC in particular is not simply:

```text
subtract speaker signal from microphone signal
```

Production acoustic echo cancellation needs to deal with:

```text
unknown acoustic impulse response
speaker/microphone delay
non-linear speaker distortion
room changes
double-talk
clock drift
device-specific gain
Bluetooth latency
dynamic routing
```

A poor custom AEC can sound worse than no AEC.

Therefore the architecture permits:

```text
Android platform AEC
vendor/hardware DSP
mature external backend
```

behind a safe Rust interface when that is the superior engineering choice.

---

# 4. High-Level Audio Architecture

```text
                    CAPTURE PATH

Microphone / Audio Device
          ↓
Native Audio I/O Adapter
          ↓
Capture Ring Buffer
          ↓
Sample Format Normalize
          ↓
Resampler / Drift Corrector
          ↓
High-Pass / DC Filter
          ↓
AEC
          ↓
Noise Suppression
          ↓
AGC / Limiter
          ↓
VAD / Level Analysis
          ↓
Opus Framing
          ↓
Opus Encoder
          ↓
Network
```

Receive:

```text
                    PLAYBACK PATH

Network
   ↓
Jitter Buffer
   ↓
Opus Decoder
   ↓
PLC / Recovery
   ↓
Playback Resampler
   ↓
Mixer
   ↓
Limiter
   ↓
Playback Ring Buffer
   ↓
Native Audio Output
   ↓
Speaker / Headset
```

AEC receives a reference from playback:

```text
Decoded/Mixed Playback Reference
             │
             └──────────────→ AEC Reference Input
```

---

# 5. Primary Architectural Split

```text
comm-audio-core
    |
    +-- pure Rust DSP
    |
    +-- audio graph
    |
    +-- timing/drift
    |
    +-- Opus integration
    |
    +-- backend traits
            |
            +-- Android AAudio/Oboe-like native backend
            +-- Android hardware/platform AEC/NS/AGC
            +-- Linux audio backend
            +-- Windows audio backend
            +-- macOS/iOS backend
```

---

# 6. Suggested Workspace

```text
crates/
├── comm-audio-core/
├── comm-audio-dsp/
├── comm-audio-resample/
├── comm-audio-aec/
├── comm-audio-ns/
├── comm-audio-agc/
├── comm-audio-vad/
├── comm-audio-jitter/
├── comm-audio-opus/
├── comm-audio-android/
├── comm-audio-linux/
├── comm-audio-windows/
├── comm-audio-apple/
├── comm-audio-diagnostics/
└── comm-audio-testkit/
```

---

# 7. `comm-audio-core`

Owns:

```text
AudioFrame
AudioFormat
AudioClock
AudioGraph
CaptureSession
PlaybackSession
AudioRoute
AudioQualityPolicy
```

It is platform-neutral.

---

# 8. Core Audio Format

Internally prefer one canonical processing format.

Recommended voice pipeline:

```text
48,000 Hz
mono
f32
```

Reasons:

```text
Opus naturally uses 48 kHz internal timing
f32 simplifies DSP
mono is normal for voice capture
```

Stereo can be supported for media/music separately.

---

# 9. Do Not Assume Microphone Is 48 kHz

Actual input may be:

```text
8 kHz
16 kHz
44.1 kHz
48 kHz
96 kHz
```

The system must normalize.

---

# 10. Audio Format Type

```rust
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: SampleFormat,
}
```

---

# 11. Sample Format

```rust
pub enum SampleFormat {
    I16,
    I24Packed,
    I32,
    F32,
}
```

Keep conversions centralized.

---

# 12. Internal Frame

```rust
pub struct AudioFrame {
    pub timestamp: AudioTimestamp,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: AudioBuffer,
}
```

---

# 13. Audio Buffer

Use pooled buffers.

Avoid:

```text
Vec allocation every 10 ms
```

---

# 14. Buffer Pool

```rust
pub struct AudioBufferPool {
    // bounded reusable frame buffers
}
```

---

# 15. Frame Duration

Voice processing should use fixed small frames.

Typical logical units:

```text
10 ms
20 ms
```

depending DSP/Opus configuration.

---

# 16. 10 ms Processing Quantum

A useful internal DSP quantum is:

```text
10 ms @ 48 kHz = 480 samples/channel
```

This aligns well with many real-time speech algorithms.

---

# 17. Opus Packet Duration

Can use:

```text
10 ms
20 ms
40 ms
```

but lower durations reduce latency at cost of packet overhead.

For interactive calling:

```text
20 ms
```

is a practical default.

---

# 18. Resampling Requirement

Example:

```text
device microphone = 44.1 kHz
internal pipeline = 48 kHz
```

Therefore:

```text
44.1 → 48 kHz
```

must be performed before Opus/DSP where necessary.

---

# 19. Resampler Goals

Must provide:

```text
good speech quality
low latency
bounded CPU
streaming operation
phase continuity
dynamic fractional ratio support
```

---

# 20. Rust Resampler

This should be implemented/hosted entirely in Rust.

No reason to use Kotlin for resampling.

Architecture:

```rust
pub trait AudioResampler {
    fn process(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        ratio: f64,
    ) -> Result<ResampleStats, AudioError>;
}
```

---

# 21. Resampling Algorithms

Possible implementations:

```text
polyphase FIR
windowed-sinc
band-limited interpolation
```

Avoid naïve:

```text
nearest neighbor
basic linear interpolation
```

for production speech quality.

---

# 22. Fixed Ratio vs Drift Ratio

Two different problems:

```text
44.1 kHz → 48 kHz
```

and:

```text
48,000.0 Hz device clock
vs
47,998.8 Hz remote/effective clock
```

The second requires continuous drift correction.

---

# 23. Clock Drift

Two devices rarely run at exactly the same sample clock.

If ignored, playback buffer eventually:

```text
underflows
or
overflows
```

---

# 24. Drift Estimator

```rust
pub struct AudioClockDriftEstimator {
    // rolling error estimate
}
```

Inputs:

```text
buffer occupancy
capture timestamps
playback timestamps
remote RTP/media timestamps
```

---

# 25. Fractional Resampling for Drift

Adjust ratio slightly:

```text
0.99995
1.00003
```

instead of dropping large chunks of audio.

---

# 26. Drift Correction Policy

Use:

```text
small continuous ratio corrections
```

first.

Only use:

```text
sample insertion/drop
```

as emergency correction.

---

# 27. Capture Clock

Use monotonic timestamps from audio backend where available.

---

# 28. Playback Clock

Track actual consumed frames, not just submitted frames.

---

# 29. Audio I/O Backend Trait

```rust
pub trait AudioDeviceBackend {
    fn start_capture(&mut self, config: CaptureConfig) -> Result<(), AudioError>;
    fn start_playback(&mut self, config: PlaybackConfig) -> Result<(), AudioError>;
    fn stop_capture(&mut self);
    fn stop_playback(&mut self);
}
```

Actual implementation should support real-time callbacks/ring buffers.

---

# 30. Callback Rule

Platform audio callback must do minimal work.

Do not execute:

```text
network
allocation-heavy DSP
database
logging formatting
```

inside real-time audio callback.

---

# 31. Callback Architecture

```text
Audio callback
   ↓
lock-free/SPSC ring buffer
   ↓
Rust audio worker
   ↓
DSP graph
```

Playback:

```text
DSP worker
   ↓
playback ring buffer
   ↓
audio callback
```

---

# 32. Real-Time Safety

The audio callback should avoid:

```text
blocking mutex
heap allocation
sleep
file I/O
network I/O
```

---

# 33. Ring Buffer

Use bounded:

```text
single producer / single consumer
```

ring where topology permits.

---

# 34. Capture Ring

If full:

```text
drop oldest/newest according to real-time policy
```

Do not block microphone callback.

For voice, freshness normally wins.

---

# 35. Playback Ring

If empty:

```text
output silence
```

and increment underrun counter.

Never block callback waiting for network.

---

# 36. Android Audio I/O

Prefer low-latency Android native audio facilities.

Rust should call platform native API directly where practical.

Avoid:

```text
Kotlin AudioRecord ByteArray loop
```

for the hot path.

---

# 37. Android Native Backend

Recommended crate:

```text
comm-audio-android
```

Responsibilities:

```text
stream open
stream start/stop
device routing
native callback
audio format discovery
latency reporting
hardware DSP session binding
```

---

# 38. Platform API Boundary

If Android NDK audio API is sufficient:

```text
call directly from Rust
```

through a small unsafe wrapper.

If a platform feature is only conveniently accessible through Java/Kotlin:

```text
use a narrow control bridge
```

but never send PCM frames through it.

---

# 39. Capture Data Plane

Correct:

```text
Android audio callback
→ native Rust ring buffer
```

Incorrect:

```text
AudioRecord
→ Kotlin ByteArray
→ JNI
→ Rust Vec
```

for every frame.

---

# 40. Playback Data Plane

Correct:

```text
Rust ring buffer
→ native Android output callback
```

---

# 41. Audio Route Types

```rust
pub enum AudioRoute {
    Speaker,
    Earpiece,
    WiredHeadset,
    BluetoothSco,
    BluetoothLeAudio,
    Usb,
    Unknown,
}
```

---

# 42. Route Change

Route may change during a call.

Examples:

```text
speaker → Bluetooth
Bluetooth → earpiece
USB headset → speaker
```

The pipeline must re-evaluate:

```text
sample rate
latency
AEC
NS
AGC
buffer sizing
```

---

# 43. Route Event

```rust
pub struct AudioRouteChanged {
    pub old: AudioRoute,
    pub new: AudioRoute,
    pub new_format: Option<AudioFormat>,
}
```

---

# 44. Bluetooth Latency

Bluetooth can add significant and variable latency.

AEC delay estimator must account for route-dependent delay.

---

# 45. Hardware DSP Availability

Some devices expose:

```text
AEC
NS
AGC
```

through platform audio effects or DSP.

Capabilities vary.

---

# 46. DSP Backend Strategy

Each processor can have:

```text
Rust backend
Platform backend
Disabled backend
```

---

# 47. AEC Backend Trait

```rust
pub trait EchoCanceller {
    fn process_capture(
        &mut self,
        mic: &mut [f32],
        reference: &[f32],
        timing: EchoTiming,
    ) -> Result<EchoStats, AudioError>;
}
```

---

# 48. Hardware AEC Adapter

For platform AEC:

```text
Rust owns session/policy
platform owns actual DSP
```

The trait may operate differently because hardware AEC can be inserted into the capture stream rather than accepting explicit frame buffers.

Therefore define backend capability:

```rust
pub enum AecMode {
    InlineSoftware,
    PlatformStreamEffect,
}
```

---

# 49. AEC Selection Policy

Recommended:

```text
known-good hardware/platform AEC
→ prefer platform

else mature Rust/software AEC backend
→ use software

else
→ echo control degraded / headset recommendation
```

---

# 50. Do Not Blindly Prefer Hardware

Some device AEC implementations can be poor or broken.

Maintain:

```text
capability probe
quirk policy
diagnostics
```

---

# 51. AEC Reference Signal

Software AEC needs the signal actually sent to output, ideally:

```text
post-mix
post-volume model where practical
```

---

# 52. Playback Reference Tap

```text
Opus decode
 ↓
mix
 ↓
limiter
 ├──→ speaker ring
 └──→ AEC reference history
```

---

# 53. Reference Delay

AEC needs alignment between:

```text
speaker reference
mic capture
```

---

# 54. Delay Estimator

```rust
pub struct EchoDelayEstimator {
    // estimated acoustic/system delay
}
```

Inputs:

```text
audio backend latency
ring occupancy
correlation estimates
route type
```

---

# 55. Double-Talk

AEC must not destroy near-end speech when both users talk simultaneously.

This is one reason production AEC is complex.

---

# 56. AEC Metrics

Track:

```text
estimated delay
echo return loss
echo suppression
double-talk
backend
```

where backend exposes them.

---

# 57. Noise Suppression

NS reduces:

```text
fan noise
traffic
air conditioner
constant room noise
```

---

# 58. Rust NS

A basic/high-quality spectral noise suppressor can be implemented in Rust.

Possible architecture:

```text
frame
→ STFT
→ noise estimator
→ gain mask
→ inverse STFT
```

---

# 59. FFT

FFT implementation should be Rust-native where practical.

No need for Kotlin.

---

# 60. NS Complexity Ladder

```text
Level 0: off
Level 1: simple spectral gate
Level 2: classical noise suppression
Level 3: ML-based suppressor
```

Start with classical Rust DSP unless product quality demands more.

---

# 61. ML Noise Suppression

Potential later option:

```text
small neural suppressor
```

but introduces:

```text
model runtime
CPU
battery
latency
```

Should be optional.

---

# 62. Platform NS

If Android device offers good hardware NS:

```text
may prefer it
```

to reduce CPU.

---

# 63. Avoid Double Noise Suppression

Do not run:

```text
hardware NS
+
software strong NS
```

without deliberate tuning.

It can make speech metallic.

---

# 64. AGC

Automatic Gain Control keeps voice level usable.

Goals:

```text
raise quiet speech
avoid clipping
avoid pumping noise
```

---

# 65. Rust AGC

AGC is suitable for Rust implementation.

Pipeline:

```text
level detector
→ target level
→ attack/release smoothing
→ gain
→ limiter
```

---

# 66. AGC Types

```text
analog AGC
digital AGC
```

On most app-level pipelines:

```text
digital AGC
```

is easier/portable.

---

# 67. Hardware AGC

If platform modifies microphone gain effectively:

```text
can be used
```

but exact behavior varies.

---

# 68. Avoid Multiple AGCs

Do not enable:

```text
platform AGC
+
strong software AGC
```

blindly.

---

# 69. Limiter

Always include a final soft limiter to prevent clipping after:

```text
AGC
mixing
effects
```

---

# 70. High-Pass Filter

Voice capture benefits from mild high-pass/DC removal.

Can reduce:

```text
handling noise
DC offset
very low-frequency rumble
```

Pure Rust.

---

# 71. DC Blocker

Simple first-order filter.

Pure Rust.

---

# 72. VAD

Voice Activity Detection can drive:

```text
UI speaking indicator
silence suppression
noise estimator
bandwidth policy
```

---

# 73. Rust VAD

Start with:

```text
energy
spectral features
hangover
```

Pure Rust.

---

# 74. VAD Is Not Authentication

Do not infer:

```text
who is speaking
```

only speech activity.

---

# 75. Silence Suppression

Optional.

If VAD says silence:

```text
Opus DTX
```

can reduce bandwidth.

---

# 76. Opus DTX

Preferred to fully stopping media timeline.

Keeps timing semantics cleaner.

---

# 77. Opus FEC

Enable in degraded networks where useful.

Part of Opus/network policy.

---

# 78. Packet-Loss Concealment

Opus decoder provides PLC behavior.

Jitter/receiver policy decides when to:

```text
wait
FEC recover
PLC
```

---

# 79. Jitter Buffer

Dedicated Rust subsystem.

```rust
pub struct AudioJitterBuffer {
    // ordered timestamped packet buffer
}
```

---

# 80. Jitter Goals

Balance:

```text
latency
loss recovery
stability
```

---

# 81. Adaptive Jitter

Target delay adapts to:

```text
network jitter
loss
route changes
```

---

# 82. Minimum Buffer

LAN/direct:

```text
small
```

Relay/cellular:

```text
larger
```

---

# 83. Jitter Buffer Bounds

Never grow without limit.

Example policy:

```text
20–200 ms
```

depending call mode.

Tune from measurements.

---

# 84. Late Packet

If packet arrives after playout deadline:

```text
discard
```

unless useful for FEC/state.

---

# 85. Reordering

Jitter buffer handles bounded reordering.

---

# 86. Network Handoff

Wi-Fi → cellular may suddenly alter:

```text
latency
jitter
```

Jitter buffer adapts gradually.

---

# 87. Multipath Audio

Part 12 may use:

```text
redundant path
```

but avoid excessive duplicate audio unless necessary.

---

# 88. Audio Priority

Audio frames outrank:

```text
video
bulk files
background sync
```

during calls.

---

# 89. Media Scheduler

```text
call control
audio
video keyframe
video delta
bulk
```

---

# 90. Audio Network Frame

```rust
pub struct EncodedAudioFrame {
    pub sequence: u64,
    pub timestamp: AudioTimestamp,
    pub duration_samples: u16,
    pub opus_payload: Bytes,
}
```

---

# 91. Sequence Number

Used for:

```text
loss
reordering
diagnostics
```

---

# 92. Timestamp

Use media sample timeline.

Not wall clock.

---

# 93. Opus Encoder Input

Always provide normalized:

```text
48 kHz mono/stereo
```

as configured.

---

# 94. Capture Resampler Placement

Recommended:

```text
native capture format
→ normalize to f32
→ resample to 48 kHz
→ DSP
→ Opus
```

Some AEC backends may require different placement.

Backend declares requirements.

---

# 95. DSP Graph

Use explicit graph/pipeline rather than hardcoded call chain.

```rust
pub struct AudioProcessingGraph {
    // ordered processing nodes
}
```

---

# 96. DSP Node

```rust
pub trait AudioProcessor {
    fn process(
        &mut self,
        frame: &mut AudioFrame,
        ctx: &AudioProcessContext,
    ) -> Result<(), AudioError>;
}
```

---

# 97. Graph Example

```text
FormatConvert
→ Resample
→ HighPass
→ AEC
→ NS
→ AGC
→ Limiter
→ VAD
```

---

# 98. Graph Reconfiguration

When route changes:

```text
hardware AEC available
```

graph can become:

```text
Resample
→ HighPass
→ NS
→ light AGC
```

with AEC outside graph in platform stream.

---

# 99. Processor Latency

Every DSP node declares:

```text
algorithmic latency
```

---

# 100. Latency Budget

End-to-end audio latency includes:

```text
capture buffer
DSP
Opus frame
network
jitter
decode
playback buffer
hardware
```

---

# 101. Audio Latency Budget Model

```rust
pub struct AudioLatencyBudget {
    pub capture_ms: f32,
    pub dsp_ms: f32,
    pub codec_ms: f32,
    pub network_ms: f32,
    pub jitter_ms: f32,
    pub playback_ms: f32,
}
```

---

# 102. Target

Interactive voice should aim for the lowest stable latency possible.

Avoid adding DSP that costs:

```text
100+ ms
```

unless quality benefit justifies it.

---

# 103. Processing Deadline

A 10 ms DSP frame must finish comfortably before next frame arrives.

---

# 104. CPU Budget

Part 08 limits audio worker CPU indirectly through policy.

Audio is real-time priority but must remain efficient.

---

# 105. Thread Priority

Platform audio callback may use elevated real-time audio scheduling.

DSP worker can use high priority carefully.

Do not starve whole app.

---

# 106. No Rayon in Real-Time Callback

General Rayon pool is inappropriate inside hard audio callback.

Dedicated worker/pipeline is better.

---

# 107. SIMD

Rust DSP may use portable SIMD where mature/stable for target.

Always maintain scalar fallback.

---

# 108. FFT Plan Reuse

Precompute/reuse FFT plans.

Do not allocate/plan every frame.

---

# 109. DSP State Reuse

Filters maintain state across frames.

Never recreate each 10 ms block.

---

# 110. Denormals

Floating-point DSP should avoid pathological denormal performance where relevant.

---

# 111. Numeric Stability

Use:

```text
f32
```

for real-time DSP unless a specific algorithm needs f64.

---

# 112. Fixed-Point

Only use fixed-point if:

```text
embedded target
measured benefit
```

justifies complexity.

---

# 113. Android Hardware DSP Backend

Possible architecture:

```text
Rust AudioSession
   ↓
Android audio stream/session ID
   ↓
platform AEC/NS/AGC effect
```

Control calls may require:

```text
small JNI/platform bridge
```

if no suitable NDK API is available.

PCM still remains native.

---

# 114. Narrow JNI Rule

Allowed:

```text
enable effect
disable effect
query availability
bind audio session
```

Not allowed in hot path:

```text
send every PCM frame
```

---

# 115. Rust Owns Decision

Kotlin should not decide:

```text
AEC policy
NS aggressiveness
AGC mode
```

Rust media policy does.

---

# 116. DSP Capability Model

```rust
pub struct AudioDspCapabilities {
    pub platform_aec: bool,
    pub platform_ns: bool,
    pub platform_agc: bool,
    pub low_latency_io: bool,
    pub hardware_sample_rate: u32,
}
```

---

# 117. Backend Selection

```rust
pub enum ProcessorBackend {
    Rust,
    Platform,
    Disabled,
}
```

---

# 118. Audio Processing Profile

```rust
pub struct AudioProcessingProfile {
    pub aec: ProcessorBackend,
    pub ns: ProcessorBackend,
    pub agc: ProcessorBackend,
    pub vad: bool,
}
```

---

# 119. Auto Profile

At call start:

```text
detect route
detect hardware DSP
check quirk DB
check battery/CPU
select profile
```

---

# 120. Speakerphone

AEC usually important.

Profile:

```text
AEC on
NS on
AGC moderate
```

---

# 121. Headphones

Echo path is much smaller.

Can reduce/disable AEC depending detection.

---

# 122. Bluetooth Headset

Headset may already perform significant DSP.

Avoid double-processing.

---

# 123. USB Professional Microphone

May not need AGC.

Allow advanced/manual profile.

---

# 124. Audio Mode

```rust
pub enum AudioUseCase {
    VoiceCall,
    VoiceMessage,
    MusicPlayback,
    Recording,
}
```

DSP differs by use case.

---

# 125. Voice Call

Needs:

```text
AEC
NS
AGC
low latency
```

---

# 126. Voice Message

No speaker reference during recording usually.

AEC unnecessary.

Can use:

```text
NS
AGC
```

---

# 127. Music Playback

Do not apply speech DSP.

---

# 128. Recording

Preserve quality.

Use effects only when requested.

---

# 129. Voice Message Path

```text
mic
→ resample
→ HPF
→ NS
→ AGC
→ Opus
→ file/blob
```

---

# 130. Call Playback Mixing

Need to mix:

```text
remote voice
local UI tones
possibly multiple participants
```

---

# 131. Mixer

Pure Rust.

```rust
pub struct AudioMixer {
    // bounded input streams
}
```

---

# 132. Mixing Headroom

Do not simply add signals and clip.

Use:

```text
gain normalization
limiter
```

---

# 133. Group Calls

For N remote participants:

```text
decode streams
→ per-stream gain
→ mix
→ limiter
→ playback
```

---

# 134. Active Speaker

VAD/audio level can inform UI.

---

# 135. Per-Peer Gain

User may adjust participant volume.

Pure Rust mixer.

---

# 136. Mute

Mute can occur:

```text
before Opus encode
```

while keeping media timing/control alive.

---

# 137. Push-to-Talk

Rust call state controls capture gate.

---

# 138. Audio Focus

Android may interrupt for:

```text
phone call
alarm
another media app
```

Platform adapter reports focus state.

---

# 139. Focus State

```rust
pub enum AudioFocusState {
    Granted,
    Duck,
    LostTransient,
    Lost,
}
```

---

# 140. Focus Policy

Rust decides:

```text
pause
duck
resume
```

according to call state.

---

# 141. Incoming Cellular Call

P2P audio may need pause/duck.

Must handle cleanly.

---

# 142. Interruptions

iOS/macOS equivalent later.

---

# 143. Device Route Diagnostics

Part 18 should display:

```text
Input: built-in microphone
Output: speaker
Capture: 48 kHz
AEC: Android platform
NS: Rust
AGC: Rust
```

---

# 144. Audio Diagnostics

Track:

```text
capture underruns/overruns
playback underruns
resampler ratio
drift ppm
jitter target
packet loss
PLC frames
FEC recoveries
AEC backend
NS backend
AGC gain
audio route
```

---

# 145. Audio Level

Expose privacy-safe:

```text
RMS/dBFS level
```

not raw samples.

---

# 146. Debug Waveform

Only explicit developer mode.

Do not log/store microphone PCM by default.

---

# 147. Privacy

Never dump:

```text
microphone audio
speaker audio
AEC reference
```

into logs.

---

# 148. Echo Diagnostics

Use aggregate metrics.

---

# 149. Ring Buffer Diagnostics

Track:

```text
fill level
high-water mark
underrun/overrun
```

---

# 150. Drift Diagnostics

Display:

```text
+18 ppm
```

in developer mode.

---

# 151. Jitter Diagnostics

Display:

```text
target 50 ms
actual 42–71 ms
```

---

# 152. Battery Policy

Audio should remain enabled longer than video.

On battery saver:

```text
keep voice quality
reduce expensive NS if necessary
disable unnecessary stereo
```

---

# 153. Critical Battery

Still preserve:

```text
voice
```

where possible.

---

# 154. Thermal Policy

If software DSP becomes expensive:

```text
reduce NS complexity
prefer platform DSP
```

---

# 155. AEC CPU Pressure

Never disable AEC silently on speakerphone without diagnostics.

If necessary:

```text
switch to earpiece/headset recommendation
```

---

# 156. Emergency Audio

Part 17:

```text
audio remains high priority
```

Video degrades first.

---

# 157. Audio Call Fallback Ladder

```text
high-quality voice
→ reduced DSP complexity
→ lower Opus bitrate
→ narrow-band voice if needed
→ voice note
→ text
```

---

# 158. Opus Bitrate Adaptation

Rust congestion controller adjusts:

```text
bitrate
FEC
DTX
packet duration
```

---

# 159. Packet Duration Changes

Changing 20 ms → 40 ms reduces packet overhead but increases latency.

Use only under poor network conditions.

---

# 160. Mono

Voice calls should default mono.

Stereo voice wastes bandwidth unless specific use case.

---

# 161. Sample-Rate Strategy

Network codec timeline remains 48 kHz.

Local device format can differ.

---

# 162. Opus Decoder Output

Prefer 48 kHz internal output.

Then resample only if playback device requires different native rate.

---

# 163. Playback Resampling

```text
Opus 48k
→ DSP/mix 48k
→ device native rate
```

---

# 164. Capture Resampling

```text
device native rate
→ 48k DSP
```

---

# 165. Avoid Multiple Resamplers

Keep one rate conversion at each device edge.

---

# 166. Channel Conversion

Capture:

```text
stereo mic
→ mono voice
```

using weighted/downmix.

Playback:

```text
mono remote
→ stereo device output
```

if required.

---

# 167. Downmix Safety

Avoid clipping.

---

# 168. Acoustic Echo Path After Resampling

AEC reference/mic should share consistent processing rate.

Prefer:

```text
48 kHz
```

if software AEC.

---

# 169. Software AEC Development Strategy

Do not begin by writing sophisticated adaptive echo cancellation from scratch.

Recommended sequence:

```text
1. define Rust trait
2. integrate platform AEC
3. add mature software backend if available
4. only develop native Rust AEC if there is a strong reason
```

---

# 170. If Native Rust AEC Is Eventually Built

Architecture would need:

```text
adaptive filter
delay estimator
double-talk detector
residual echo suppressor
non-linear processor
clock drift handling
```

This is a dedicated project by itself.

---

# 171. NS Development Strategy

Classical Rust NS is much more reasonable to implement than full AEC.

Start:

```text
STFT
noise PSD estimate
Wiener-like gain
smoothing
```

---

# 172. AGC Development Strategy

Rust AGC is straightforward enough to own.

---

# 173. Resampler Development Strategy

Either:

```text
use a mature permissive Rust resampler crate
```

or:

```text
implement dedicated high-quality streaming resampler
```

inside `comm-audio-resample`.

Prefer mature well-tested Rust implementation before writing from scratch.

---

# 174. Crate Policy

For DSP dependencies:

```text
pure Rust preferred
permissive license preferred
well-maintained
no hidden system dependencies
```

---

# 175. Avoid Premature Custom DSP

The platform's value is communication reliability, not proving every math primitive was handwritten.

---

# 176. Testing — Resampler

Test:

```text
44.1 → 48
48 → 44.1
16 → 48
48 → 16
fractional drift ratios
```

---

# 177. Resampler Quality Tests

Measure:

```text
passband ripple
alias rejection
SNR
phase continuity
```

---

# 178. Resampler Streaming Test

Chunk sizes vary.

Output should match continuous reference within tolerance.

---

# 179. Drift Test

Simulate:

```text
+100 ppm
-100 ppm
```

for hours.

Expected:

```text
ring buffer stable
no periodic clicks
```

---

# 180. AEC Test

Use synthetic:

```text
far-end speech
room impulse response
mic near-end speech
```

Measure echo reduction.

---

# 181. Double-Talk Test

Both sides speak.

AEC must preserve local voice.

---

# 182. Route Delay Test

Simulate:

```text
speaker
Bluetooth
wired headset
```

different delays.

---

# 183. NS Test

Noisy speech samples:

```text
fan
traffic
white/pink noise
```

Measure quality and intelligibility.

---

# 184. AGC Test

Input levels:

```text
-50 dBFS
-30 dBFS
-10 dBFS
```

Output should converge without clipping/pumping.

---

# 185. Jitter Test

Inject:

```text
0–100 ms variable delay
loss
reordering
```

---

# 186. Packet Loss Test

Validate:

```text
FEC
PLC
late packet discard
```

---

# 187. Playback Underrun Test

Starve network.

Expected:

```text
silence/PLC
no crash
```

---

# 188. Capture Overrun Test

Stall DSP worker.

Expected:

```text
bounded drop
diagnostic counter
```

---

# 189. Audio Callback Stress

Inject CPU load.

Audio callback must remain stable.

---

# 190. No-Allocation Callback Test

Instrument production callback.

Assert:

```text
no heap allocation in steady state
```

where practical.

---

# 191. Android Device Tests

Test:

```text
speakerphone
earpiece
wired headset
Bluetooth headset
USB audio
```

---

# 192. Android AEC Matrix

Record:

```text
platform AEC available
actual quality
device quirks
```

---

# 193. Platform DSP Quirk Database

```rust
pub struct AudioDspQuirk {
    pub device_match: DeviceMatcher,
    pub disable_aec: bool,
    pub disable_ns: bool,
    pub disable_agc: bool,
}
```

---

# 194. Real-World Echo Test

Put two devices in rooms with:

```text
speakerphone
hard surfaces
high volume
```

Measure intelligibility.

---

# 195. Bluetooth Test

Switch headset mid-call.

Pipeline reconfigures without crash.

---

# 196. Audio Focus Test

Receive cellular call/other interruption.

---

# 197. Background Test

Android activity closes while service maintains audio call where OS policy permits.

---

# 198. Screen-Off Test

Voice call continues efficiently.

---

# 199. Long Soak

Run:

```text
2–8 hour voice call
```

track:

```text
memory
drift
latency
underruns
battery
thermal
```

---

# 200. Cross-Platform Conformance

Use synthetic DSP fixtures so:

```text
Android
Linux
Windows
macOS
```

produce compatible media timing/Opus behavior.

---

# 201. Hardware DSP Difference

Exact processed PCM need not be identical across hardware backends.

Conformance checks:

```text
timing
format
stability
bounded levels
```

not bit-identical samples.

---

# 202. Fuzzing

Part 10 fuzz:

```text
audio packet parser
Opus framing glue
resampler config
DSP graph config
route events
```

---

# 203. Property Tests

Examples:

```text
resampler never emits NaN
AGC never exceeds hard limiter
ring occupancy always bounded
invalid route sequence never corrupts session
```

---

# 204. SIMD Differential Tests

Compare:

```text
SIMD
scalar
```

within tolerance.

---

# 205. DSP Determinism

Floating-point outputs may differ slightly by CPU.

Use tolerance-based tests.

---

# 206. Audio Error Model

```rust
pub enum AudioError {
    UnsupportedFormat,
    DeviceUnavailable,
    PermissionDenied,
    RouteChanged,
    BufferOverrun,
    BufferUnderrun,
    ResampleFailed,
    AecUnavailable,
    DspFailed,
    OpusFailed,
    Timeout,
    Platform,
}
```

---

# 207. Degraded Modes

If AEC unavailable:

```text
speakerphone quality warning
prefer earpiece/headset
```

If NS unavailable:

```text
continue call
```

If AGC unavailable:

```text
continue with manual/static gain
```

If resampler unavailable:

```text
call cannot start if device rate incompatible
```

but resampler is expected to always be available in Rust.

---

# 208. Audio Session State

```rust
pub enum AudioSessionState {
    Idle,
    Starting,
    Active,
    Reconfiguring,
    Interrupted,
    Degraded,
    Stopping,
    Failed,
}
```

---

# 209. Route Reconfiguration State

```text
Active
 ↓
RouteChanged
 ↓
Pause I/O briefly
 ↓
reopen backend
 ↓
reselect DSP
 ↓
prime buffers
 ↓
Active
```

---

# 210. Avoid Large Glitch

Target route change interruption:

```text
as short as platform permits
```

---

# 211. AudioGraph Builder

```rust
let graph = AudioGraph::builder()
    .resample_to(48_000)
    .high_pass(true)
    .aec(AecPolicy::Auto)
    .noise_suppression(NsPolicy::Auto)
    .agc(AgcPolicy::Auto)
    .vad(true)
    .build()?;
```

---

# 212. Backend Policy

```rust
pub enum BackendPreference {
    Auto,
    RustOnly,
    PlatformPreferred,
    PlatformOnly,
}
```

Useful for:

```text
testing
debugging
advanced deployment
```

Default:

```text
Auto
```

---

# 213. Auto Means

For each feature:

```text
use Rust if it is mature, portable, and efficient
use platform if it is substantially better or avoids unnecessary low-level complexity
```

---

# 214. Recommended Backend Ownership

## Resampling

```text
Rust
```

## Format/channel conversion

```text
Rust
```

## High-pass/DC

```text
Rust
```

## Mixer

```text
Rust
```

## Limiter

```text
Rust
```

## AGC

```text
Rust first
```

## VAD

```text
Rust first
```

## Noise suppression

```text
Rust classical first
platform optional
```

## AEC

```text
platform/mature backend first
Rust only when quality proven
```

## Audio I/O

```text
native platform API through Rust
```

---

# 215. Production Phases

## Phase 1 — Audio Core

```text
format
frames
buffer pool
ring buffers
clock
```

## Phase 2 — Pure-Rust Resampler

```text
44.1↔48
fractional drift
```

## Phase 3 — Opus Pipeline

```text
20 ms
DTX
FEC
PLC
```

## Phase 4 — Rust DSP

```text
HPF
DC
AGC
limiter
VAD
classical NS
```

## Phase 5 — Android Native I/O

```text
low latency
no Kotlin PCM path
route changes
```

## Phase 6 — AEC Backend

```text
Android platform first
software trait fallback
```

## Phase 7 — Adaptive Jitter / Drift

```text
clock compensation
network adaptation
```

## Phase 8 — Device Quirks

```text
speaker
Bluetooth
USB
platform DSP
```

## Phase 9 — Hardening

```text
soak
echo room
battery
thermal
route changes
interruptions
```

---

# 216. Initial Production Recommendation

Do **not** block the whole application waiting for a pure-Rust world-class AEC implementation.

Ship the first production audio system as:

```text
Rust:
    audio graph
    resampler
    jitter
    drift
    Opus orchestration
    mixer
    HPF/DC
    AGC
    limiter
    VAD
    optional classical NS

Android platform/native:
    low-latency audio I/O
    proven AEC when available
    optionally proven NS/AGC where device quality is better
```

Then replace platform pieces with Rust only when the Rust backend proves:

```text
equal/better quality
equal/better latency
acceptable battery cost
wide-device stability
```

---

# 217. Definition of Done

Part 26 is complete when:

- microphone and playback sample rates may differ from 48 kHz
- all voice processing is normalized to a stable internal 48 kHz timeline
- high-quality streaming resampling is implemented in Rust
- fractional resampling corrects long-term clock drift
- capture/playback hot paths avoid Kotlin `ByteArray` PCM transfers
- native audio callbacks use bounded ring buffers
- steady-state audio callbacks avoid blocking and unnecessary allocation
- Rust owns DSP graph, timing, policy, jitter, and Opus behavior
- HPF/DC removal, mixer, limiter, AGC, and VAD have Rust backends
- classical noise suppression has a Rust backend or clearly defined replacement path
- AEC has a Rust trait and can use Android platform/hardware backend when superior
- route changes re-evaluate sample rate, latency, and DSP
- Bluetooth/headset/speakerphone behavior is explicitly handled
- hardware and software DSP are not accidentally double-enabled
- adaptive jitter buffering is bounded
- FEC/PLC/DTX integrate with network policy
- audio outranks video and bulk traffic during calls
- battery/thermal policies degrade expensive DSP before breaking the call
- diagnostics expose route, resampling ratio, drift, jitter, underruns, and DSP backends
- long-duration calls remain stable without accumulating buffer drift
- real-device echo, Bluetooth, background, interruption, and soak tests pass

---

# 218. Relationship to Earlier Parts

Part 26 strengthens:

```text
07 — Capability Negotiation
08 — Resource Limits & Backpressure
09 — Crash Recovery
10 — Fuzzing / Protocol Tests
12 — Multipath
13 — Battery-Aware Scheduling
16 — Daemon / Headless Runtime
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
25 — Android Hardware Surface / Zero-Copy Video
```

Together Parts 25 and 26 establish:

```text
video:
hardware Surface data plane

audio:
Rust DSP + native low-latency I/O + hardware-aware acoustic processing
```

---

# 219. Final Architecture

```text
                         CAPTURE

Android Native Audio Input
          │
          ▼
    SPSC Ring Buffer
          │
          ▼
   Rust Audio Worker
          │
          ├── Format Normalize
          ├── Rust Resampler
          ├── Drift Correction
          ├── HPF / DC
          ├── AEC ───────────────┐
          ├── NS                 │
          ├── AGC                │
          ├── Limiter            │
          └── VAD                │
          │                      │
          ▼                      │
        Opus                     │
          │                      │
          ▼                      │
       Network                   │
                                 │
                         AEC Reference
                                 ▲
                                 │
Network                          │
  │                              │
  ▼                              │
Jitter Buffer                    │
  │                              │
  ▼                              │
Opus Decoder                     │
  │                              │
  ▼                              │
Rust Mixer / Limiter ────────────┘
  │
  ▼
Playback Resampler
  │
  ▼
SPSC Ring Buffer
  │
  ▼
Android Native Audio Output
```

---

# 220. Final Principle

The goal is not:

```text
"everything must be handwritten in Rust"
```

The goal is:

```text
Rust owns the architecture
Rust owns portable DSP
Rust owns timing
Rust owns safety
Rust owns policy
Rust owns network/media integration

platform hardware is used where it is genuinely better
```

For this audio system, the right engineering split is:

```text
Resampling        → Rust
Clock drift       → Rust
Jitter            → Rust
Mixing            → Rust
AGC               → Rust
VAD               → Rust
HPF / limiter     → Rust
Classical NS      → Rust first
AEC               → mature platform/hardware backend first
Audio I/O         → native platform API controlled from Rust
```

That gives the application a genuinely Rust-centric audio architecture without spending enormous effort rebuilding the deepest parts of Android's acoustic stack solely for language purity.
