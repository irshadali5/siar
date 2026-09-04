# Part 25 — Android Direct Hardware Surface / Zero-Copy Media Pipeline Architecture

## Reusable P2P Communication Platform

**Status:** Architecture specification  
**Part:** 25 of 27 additional production-hardening parts  
**Primary language:** Rust  
**Target:** Android  
**Primary goals:** eliminate avoidable CPU-side frame copies, use Android hardware video surfaces directly, keep codec/media policy in Rust, integrate with Dioxus without routing raw video frames through the UI bridge, support H.264/H.265/AV1 hardware codecs on Android, preserve AV1 software-codec fallback, reduce memory bandwidth/CPU/battery usage, and provide a production-quality media data plane.

---

# 1. Why This Part Is Needed

The existing architecture already defines:

```text
calls
media codecs
Android hardware codecs
FFI/platform adapters
battery scheduling
resource limits
network routing
```

but that is not enough to guarantee an efficient media pipeline.

A naïve implementation can still do:

```text
Camera
  ↓
YUV frame
  ↓
Java/Kotlin ByteArray
  ↓
JNI copy
  ↓
Rust Vec<u8>
  ↓
encoder input copy
```

and on receive:

```text
decoder output
  ↓
CPU buffer
  ↓
JNI ByteArray
  ↓
Rust/Kotlin copy
  ↓
UI bitmap/texture
  ↓
screen
```

At:

```text
1080p
60 fps
4K
high bit-depth
```

this wastes:

```text
memory bandwidth
CPU cycles
battery
thermal headroom
latency
```

The production design should instead use:

```text
camera / GPU surface
       ↓
hardware encoder surface
       ↓
compressed packets only
       ↓
network
       ↓
hardware decoder
       ↓
display surface
```

The raw frame should remain inside Android's hardware/native graphics pipeline whenever possible.

---

# 2. Core Principle

> **Raw video frames should not cross the Kotlin ↔ Rust boundary during the normal hardware-codec path.**

Rust controls:

```text
codec selection
session state
networking
rate adaptation
timestamps
packetization
recovery
surface lifecycle state
```

Android hardware owns:

```text
camera buffers
codec input surface
codec output surface
GPU/display buffers
```

Only compressed media/control metadata crosses the Rust networking layer.

---

# 3. Desired End-to-End Send Path

```text
Camera HAL
   ↓
Android Surface / BufferQueue
   ↓
Hardware Encoder Input Surface
   ↓
MediaCodec / NDK MediaCodec
   ↓
Compressed H.264 / H.265 / AV1 access units
   ↓
Rust media packetizer
   ↓
Rust transport / Iroh
```

No CPU-copy of raw camera frames in the normal path.

---

# 4. Desired End-to-End Receive Path

```text
Iroh / media transport
   ↓
Rust depacketizer
   ↓
Compressed access units
   ↓
Android hardware decoder
   ↓
Decoder Output Surface
   ↓
ANativeWindow / Surface
   ↓
Display compositor
```

Again:

```text
no raw decoded frame copy into Rust
```

unless explicitly needed for:

```text
screenshot
analysis
effects
recording transform
fallback renderer
```

---

# 5. Architecture Boundary

```text
                     RUST CORE

 call state
 codec policy
 bitrate controller
 congestion control
 packetization
 depacketization
 timestamps
 keyframe requests
 codec lifecycle
 diagnostics

                         │
                         │ Android Media Adapter
                         ▼

                ANDROID NATIVE MEDIA APIs

 camera
 MediaCodec / NDK MediaCodec
 Surface
 ANativeWindow
 native hardware buffers
 display compositor
```

---

# 6. "Pure Rust" Meaning

The architecture should maximize Rust ownership, but Android hardware codecs are Android platform services.

Therefore "Rust implementation" means:

```text
Rust controls the system
Rust calls Android NDK/C APIs directly
Rust owns state machines
Rust owns resource safety wrappers
Rust owns media/network policy
```

It does **not** mean reimplementing:

```text
MediaCodec
SurfaceFlinger
Camera HAL
GPU driver
```

in Rust.

Those remain Android platform components.

---

# 7. Avoid Kotlin for Media Hot Path

Kotlin may still be present for:

```text
Android application lifecycle
permissions
Dioxus bootstrap
Play Store integration
```

but the high-bandwidth video frame path must avoid:

```text
ByteArray
Bitmap
Image plane copy
JNI array copy
```

---

# 8. Rust Android Media Crate

Recommended:

```text
crates/comm-media-android/
```

Responsibilities:

```text
codec discovery
codec configuration
surface creation/binding
ANativeWindow ownership
hardware encode/decode
buffer queue polling
compressed access-unit extraction
presentation timestamp handling
codec reset/reconfigure
```

---

# 9. Crate Structure

```text
crates/comm-media-android/
├── src/
│   ├── lib.rs
│   ├── codec/
│   │   ├── mod.rs
│   │   ├── discovery.rs
│   │   ├── encoder.rs
│   │   ├── decoder.rs
│   │   ├── config.rs
│   │   └── quirks.rs
│   ├── surface/
│   │   ├── mod.rs
│   │   ├── native_window.rs
│   │   ├── input_surface.rs
│   │   ├── output_surface.rs
│   │   └── lifecycle.rs
│   ├── camera/
│   │   ├── mod.rs
│   │   ├── session.rs
│   │   └── capture.rs
│   ├── buffer/
│   │   ├── compressed.rs
│   │   ├── pool.rs
│   │   └── timestamp.rs
│   ├── capability.rs
│   ├── diagnostics.rs
│   ├── error.rs
│   └── ffi.rs
└── Cargo.toml
```

---

# 10. Native API Preference

Prefer native Android APIs where sufficient:

```text
NDK MediaCodec APIs
ANativeWindow
AHardwareBuffer where appropriate
native camera APIs where stable enough
```

Rust accesses them through:

```text
bindgen-generated bindings
existing safe Rust Android/NDK crates where appropriate
small audited unsafe wrapper modules
```

---

# 11. Unsafe Code Policy

All raw Android C API calls should remain isolated.

Example:

```text
ffi.rs
native_window.rs
codec/sys.rs
```

Safe wrappers expose Rust types.

Never spread:

```rust
unsafe
```

through call/media business logic.

---

# 12. Surface Ownership Type

Conceptually:

```rust
pub struct NativeSurface {
    raw: NonNull<ANativeWindow>,
}
```

Responsibilities:

```text
acquire
release
thread-safe ownership rules
lifetime tracking
```

---

# 13. RAII

Use Rust RAII so:

```text
surface reference acquired
→ wrapper owns it
→ Drop releases exactly once
```

No manual ownership scattered through application code.

---

# 14. Surface Handle Identity

```rust
pub struct SurfaceId(u64);
```

Use stable local IDs in Rust state machines.

Do not use raw pointer addresses as domain identifiers.

---

# 15. Surface Lifecycle

```rust
pub enum SurfaceState {
    Unavailable,
    Creating,
    Ready,
    Attached,
    Replacing,
    Releasing,
    Lost,
}
```

---

# 16. UI Surface Can Disappear

On Android:

```text
activity recreated
rotation
app backgrounded
Dioxus view removed
```

may destroy/recreate the output surface.

Decoder must not assume permanent surface lifetime.

---

# 17. Surface Replacement

State flow:

```text
Old Surface
   ↓
new Dioxus/native view arrives
   ↓
create new native surface
   ↓
rebind decoder safely
   ↓
release old surface
```

If direct rebinding is unsupported/unsafe:

```text
drain
recreate decoder
request keyframe
resume
```

---

# 18. Never Keep Dangling Surface Pointer

The Rust wrapper must ensure that:

```text
Android surface destroyed
```

causes:

```text
surface invalidation
```

before decoder writes to stale memory.

---

# 19. Hardware Encoder Input Modes

Two main modes:

```text
byte-buffer input
surface input
```

For camera/video call:

```text
surface input preferred
```

---

# 20. Surface Input Encoder

Flow:

```text
configure encoder
   ↓
request/create input Surface
   ↓
camera/GPU writes to Surface
   ↓
encoder produces compressed output
```

This is the main zero-copy architecture.

---

# 21. Camera to Encoder Surface

Preferred architecture:

```text
Camera capture session
target =
hardware encoder input Surface
```

Optional secondary target:

```text
local preview Surface
```

Camera can feed both through Android's buffer/graphics stack.

---

# 22. Local Preview

Do not obtain CPU frame just to show self-preview.

Use:

```text
camera → preview Surface
```

directly.

---

# 23. Dual-Surface Camera Session

Potential:

```text
Camera
 ├── local preview surface
 └── encoder input surface
```

Rust owns session configuration/lifecycle.

---

# 24. GPU Transform Path

If future features need:

```text
rotation
crop
blur
background effect
color transform
```

prefer:

```text
camera Surface
→ GPU
→ encoder Surface
```

instead of CPU pixel copying.

---

# 25. Raw CPU Frame Path Is Fallback

Only use CPU frames when:

```text
hardware surface path unsupported
software AV1 encoding selected
computer vision requires CPU data
special processing requires it
```

This path must be explicit and measurable.

---

# 26. Hardware Decoder Output

Configure decoder with:

```text
output Surface
```

Then:

```text
compressed bitstream
→ decoder
→ Surface
```

No decoded YUV buffer enters Rust.

---

# 27. Network Boundary

Rust networking should handle only:

```text
compressed access units
RTP-like packet payloads / custom media frames
timestamps
sequence numbers
codec config
keyframe metadata
```

---

# 28. Compressed Buffer Ownership

Unlike raw frames, compressed frames are relatively small.

A bounded copy may be acceptable.

Still prefer buffer reuse.

---

# 29. Compressed Buffer Pool

```rust
pub struct EncodedBufferPool {
    // bounded reusable buffers
}
```

Avoid:

```text
Vec allocation per encoded frame
```

---

# 30. Access Unit

```rust
pub struct EncodedAccessUnit {
    pub codec: VideoCodec,
    pub pts: MediaTimestamp,
    pub keyframe: bool,
    pub data: EncodedBuffer,
}
```

---

# 31. Codec Types

```rust
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}
```

Android hardware path supports codecs based on runtime device capability.

---

# 32. AV1 Software Fallback

Project requirement:

```text
AV1 is the only software video codec
```

Therefore:

```text
hardware H.264
hardware H.265
hardware AV1
```

on Android when available.

Fallback:

```text
software AV1
```

when compatible/viable.

Do not add software H.264/H.265 unless policy is later changed.

---

# 33. Codec Selection Ladder

Example:

```text
Peer supports AV1
+
Android hardware AV1 available
→ hardware AV1

else peer supports H.265
+
hardware H.265 available
→ hardware H.265

else peer supports H.264
+
hardware H.264 available
→ hardware H.264

else AV1 software viable
→ software AV1

else
→ video unavailable / audio-only fallback
```

Exact preference may vary by:

```text
battery
device
latency
peer support
```

---

# 34. Do Not Assume AV1 Hardware

Capability must be runtime-detected.

Same app binary runs on many Android devices.

---

# 35. Hardware Capability Model

```rust
pub struct AndroidVideoCapability {
    pub codec: VideoCodec,
    pub encode: bool,
    pub decode: bool,
    pub surface_input: bool,
    pub surface_output: bool,
    pub profiles: Vec<CodecProfile>,
    pub max_resolution: Option<Resolution>,
    pub max_fps: Option<u32>,
}
```

---

# 36. Capability Cache

Codec capability probing can be cached per:

```text
device
OS build
app version
```

but must tolerate invalidation after OS update.

---

# 37. Runtime Probe

On first use:

```text
probe actual codec
```

because advertised capability may not guarantee reliable operation.

---

# 38. Device Quirks

Android codec implementations vary.

Maintain a bounded quirk database:

```rust
pub struct CodecQuirk {
    pub device_match: DeviceMatcher,
    pub rule: CodecQuirkRule,
}
```

---

# 39. Quirk Examples

```text
disable specific codec profile
limit resolution
require decoder recreation after surface loss
avoid dynamic bitrate update
```

---

# 40. Quirk Data Must Be Explainable

Diagnostics should report:

```text
AV1 hardware disabled by known device workaround
```

not silently fail.

---

# 41. Dynamic Bitrate

Rust congestion controller should be able to request:

```text
new target bitrate
```

without tearing down encoder where Android codec supports it.

---

# 42. Bitrate Controller

Inputs:

```text
network throughput
loss
RTT
battery
thermal
receiver feedback
```

Output:

```text
target bitrate
target resolution
target fps
```

---

# 43. Resolution Adaptation

If bitrate drops significantly:

```text
reconfigure encoder
```

using safe state machine.

Potential ladder:

```text
1080p
720p
540p
360p
```

---

# 44. FPS Adaptation

Example:

```text
30 fps
24 fps
15 fps
```

depending network/power.

---

# 45. Avoid Frequent Reconfiguration

Use hysteresis.

Do not:

```text
recreate codec every second
```

due to noisy bandwidth estimates.

---

# 46. Keyframe Control

Rust media controller needs:

```text
request IDR/keyframe
```

when:

```text
decoder joins
packet loss recovery
transport switch
decoder reset
surface recreated
```

---

# 47. Decoder Recovery

On corruption:

```text
flush/restart decoder
request keyframe
```

rather than forwarding bad state indefinitely.

---

# 48. Media Timestamp

Use monotonic clock domain.

```rust
pub struct MediaTimestamp(pub i64);
```

Do not use wall clock for frame pacing.

---

# 49. Timestamp Conversion

Android codec timestamps may use microseconds.

Normalize in one module.

Never mix:

```text
ns
us
ms
```

implicitly.

---

# 50. A/V Synchronization

Video timestamps must align with:

```text
audio media clock
```

handled by media synchronization layer.

Surface rendering still respects presentation timestamps.

---

# 51. Decoder Render Timing

Do not render all decoded frames immediately.

Use:

```text
presentation scheduling
late-frame policy
```

---

# 52. Late Frame Dropping

If frame is too late:

```text
drop before display
```

where platform decoder/surface flow allows.

Low latency matters more than showing stale video.

---

# 53. Jitter Buffer

Network media subsystem maintains bounded jitter buffer before decoder.

The Surface itself is not a network jitter buffer.

---

# 54. Receive Flow

```text
network packets
 ↓
reassembly
 ↓
jitter buffer
 ↓
encoded access unit
 ↓
hardware decoder
 ↓
surface
```

---

# 55. Send Flow

```text
camera
 ↓
encoder surface
 ↓
hardware encoder
 ↓
encoded access unit
 ↓
packetizer
 ↓
network
```

---

# 56. Zero-Copy Definition

Be precise.

The architecture is not necessarily literally zero memory copies everywhere.

It aims for:

```text
zero CPU copies of raw video frames across app-language boundaries
```

Android/driver/GPU internals may still perform implementation-specific buffer operations.

---

# 57. Hardware Buffer Path

Where advanced rendering requires it, use:

```text
AHardwareBuffer
```

for native GPU-compatible buffers.

---

# 58. AHardwareBuffer Use Cases

Potential:

```text
GPU effects
cross-API texture import
native rendering integration
```

Do not add it merely for theoretical optimization.

---

# 59. Direct Surface Is Simpler

For ordinary video call receive:

```text
decoder → Surface
```

is preferable to manually consuming `AHardwareBuffer`.

---

# 60. Dioxus Integration

Dioxus should own:

```text
layout
call controls
video view placement
visibility
```

but not raw video bytes.

---

# 61. Video View Bridge

Dioxus component creates/owns a platform video surface container.

It exposes to Rust:

```text
SurfaceCreated(SurfaceHandle)
SurfaceChanged(...)
SurfaceDestroyed(SurfaceId)
```

---

# 62. No Frame Prop

Bad:

```text
Dioxus prop = Vec<u8> frame
```

Good:

```text
Dioxus prop = VideoRendererId
```

Native surface renderer updates independently.

---

# 63. Renderer Handle

```rust
pub struct VideoRendererId(u64);
```

UI binds renderer to platform surface.

Media runtime sends video to renderer internally.

---

# 64. Dioxus Recomposition

UI recomposition must not:

```text
restart decoder
copy frame
block codec thread
```

---

# 65. Orientation

Handle:

```text
portrait
landscape
sensor rotation
remote rotation metadata
```

prefer native transform/display matrix.

---

# 66. Aspect Ratio

UI can request:

```text
fit
fill
crop
```

Surface/GPU renderer applies transform without CPU frame conversion.

---

# 67. Picture-in-Picture

Android PiP should reuse same renderer/session state.

No codec restart merely because UI mode changed when avoidable.

---

# 68. Background Video Policy

When app backgrounded:

```text
disable outgoing camera video
or
continue only where OS/user policy permits
```

Audio may continue.

---

# 69. Surface Loss in Background

Decoder may:

```text
pause rendering
keep compressed receive state
```

or recreate when surface returns.

Avoid decoding useless frames if no render target and recording not active.

---

# 70. Decoder Without Surface

Policy:

```text
if video invisible
→ pause decoder / drop encoded video after necessary control
```

to save power.

---

# 71. Camera Lifecycle

Camera should be active only when:

```text
call video enabled
preview needed
```

---

# 72. Camera Permission

Android permission integration can remain platform bootstrap.

Rust receives typed state:

```rust
pub enum CameraPermission {
    Granted,
    Denied,
    NeedsUserAction,
}
```

---

# 73. Camera Hot Path

Permission/UI logic may involve Kotlin/Dioxus.

Frame data does not.

---

# 74. Encoder Threading

Do not run codec polling on UI thread.

Use dedicated asynchronous/blocking worker strategy.

---

# 75. Decoder Threading

Same.

---

# 76. Codec Worker

```rust
pub struct CodecWorker {
    // codec state + bounded channels
}
```

One codec instance should not spawn unbounded per-frame tasks.

---

# 77. Backpressure

If network cannot consume encoded frames:

```text
drop non-key video frames
reduce bitrate
reduce fps
```

Do not build unbounded queue.

---

# 78. Never Queue Seconds of Live Video

Interactive calls prioritize freshness.

Set strict encoded-frame queue limits.

---

# 79. Encoder Output Queue

Example:

```text
2–6 frames
```

depending architecture.

Tune by benchmarks.

---

# 80. Decoder Input Queue

Bound by:

```text
jitter target
latency budget
```

---

# 81. Memory Budget

Part 08 classifies:

```text
codec buffers
jitter buffer
compressed pool
surface metadata
```

---

# 82. Memory Pressure

Under pressure:

```text
reduce video resolution
reduce buffer pool
disable background preview
```

before core messaging.

---

# 83. Battery Policy

Part 13 affects:

```text
codec choice
resolution
fps
camera
multipath
```

---

# 84. Battery Saver

Possible:

```text
720p
24 fps
hardware codec mandatory
```

Avoid software AV1 when hardware codec available.

---

# 85. Critical Battery

Prefer:

```text
audio only
```

or low-resolution video.

---

# 86. Thermal Policy

If severe thermal state:

```text
reduce resolution/fps
disable software AV1
prefer hardware codec
```

---

# 87. Hardware Codec Preference

Even if software AV1 has better compression, battery/thermal cost may be unacceptable on mobile.

Codec policy should score:

```text
hardware acceleration
power
peer compatibility
bandwidth
quality
```

---

# 88. Software AV1 Input

When falling back to software AV1, raw frame access is required.

This becomes a different pipeline:

```text
camera
 ↓
native image/buffer
 ↓
Rust-accessible YUV
 ↓
software AV1
```

---

# 89. Software Fallback Copy Minimization

Even here:

```text
avoid Kotlin ByteArray
```

Prefer native buffer mapping/direct access from Rust.

---

# 90. Separate Software Pipeline

Implement:

```text
HardwareSurfacePipeline
SoftwareFramePipeline
```

behind common trait.

---

# 91. Video Pipeline Trait

```rust
pub trait VideoEncodePipeline {
    async fn start(&mut self, config: EncodeConfig) -> Result<(), MediaError>;
    async fn reconfigure(&mut self, change: EncodeChange) -> Result<(), MediaError>;
    async fn stop(&mut self) -> Result<(), MediaError>;
}
```

---

# 92. Hardware Encoder Implementation

```text
AndroidSurfaceEncoder
```

---

# 93. Software Encoder Implementation

```text
RustAv1SoftwareEncoder
```

---

# 94. Decode Pipeline Trait

```rust
pub trait VideoDecodePipeline {
    async fn attach_surface(&mut self, surface: SurfaceHandle) -> Result<(), MediaError>;
    async fn push(&mut self, frame: EncodedAccessUnit) -> Result<(), MediaError>;
    async fn stop(&mut self) -> Result<(), MediaError>;
}
```

---

# 95. Hardware Decoder

```text
AndroidSurfaceDecoder
```

---

# 96. Software Decoder

If AV1 software decode is needed:

```text
Rust AV1 decoder
+
native rendering path
```

but Android hardware decode should be preferred whenever available.

---

# 97. Codec Negotiation

Part 07 exchanges:

```text
codec
profile
level
bit depth
resolution
fps
```

---

# 98. Hardware Is Local Policy

Do not advertise:

```text
"hardware encoder"
```

as wire requirement.

Peer only needs codec compatibility.

Whether codec is hardware/software is local implementation detail.

---

# 99. Codec Configuration

H.264:

```text
profile
level
SPS/PPS
```

H.265:

```text
VPS/SPS/PPS
```

AV1:

```text
sequence header / codec config
```

Need correct transport configuration framing.

---

# 100. Parameter Sets

Send/update:

```text
codec configuration
```

when:

```text
encoder starts
resolution changes
codec restarts
```

---

# 101. Codec Reconfigure Event

Transport layer needs explicit:

```text
VideoConfigChanged
```

rather than hoping decoder infers all changes.

---

# 102. Decoder Reconfigure

If incompatible dynamic change:

```text
drain
recreate decoder
request keyframe
```

---

# 103. Network Packetization

Part of common media transport, not Android layer.

Android layer produces:

```text
codec access units
```

Transport decides packet boundaries.

---

# 104. MTU

Large encoded frame must be fragmented at media protocol layer.

Surface pipeline does not care.

---

# 105. Loss Recovery

Video recovery may use:

```text
keyframe request
selective retransmission for key data
FEC later
```

---

# 106. Keyframe Priority

Keyframe packets may receive higher transport priority than disposable delta frames.

---

# 107. Audio Priority

Audio should still outrank video for conversational quality.

---

# 108. Emergency Calls

Part 17 degradation ladder:

```text
video
→ low video
→ audio
→ voice note
→ text
```

---

# 109. Recording

If user records call:

```text
store compressed stream
```

where possible.

Avoid decode/re-encode.

---

# 110. Local Recording Path

```text
encoded outgoing access units
+
encoded incoming access units
```

can feed container/muxer.

Need sync/timestamps.

---

# 111. Screenshot

Screenshot may require one decoded frame.

Use explicit slow path.

Do not keep CPU-copy pipeline active continuously just for possible screenshot.

---

# 112. Video Effects

For effects:

```text
GPU Surface pipeline
```

preferred.

CPU effect path only if unavoidable.

---

# 113. Camera Switching

Front ↔ rear:

```text
camera session reconfiguration
```

ideally encoder remains configured when resolution/format compatible.

---

# 114. Surface Producer Switch

Bind new camera source to existing encoder Surface where possible.

---

# 115. Call Hold

On hold:

```text
stop camera
pause encoder
preserve call session
```

---

# 116. Video Mute

Video mute:

```text
stop/pause camera frames
send control state
```

not necessarily destroy entire codec immediately.

---

# 117. Screen Sharing

Future:

```text
screen capture Surface
→ encoder Surface
```

same architecture.

---

# 118. Screen Share Zero-Copy

Use platform virtual-display/surface mechanisms rather than screenshot ByteArrays.

---

# 119. Multiple Remote Videos

Group call may decode multiple streams.

Resource policy limits:

```text
max hardware decoders
visible streams
resolution per tile
```

---

# 120. Visible-Only High Quality

For grid:

```text
foreground speaker → high resolution
small tiles → low resolution
offscreen → pause
```

---

# 121. Decoder Pool

Hardware decoder count is finite.

Use admission control.

---

# 122. Codec Resource Failure

If hardware codec allocation fails:

```text
reduce active streams
fallback
audio-only
```

not crash.

---

# 123. Codec Error Model

```rust
pub enum AndroidCodecError {
    Unsupported,
    ConfigureFailed,
    SurfaceUnavailable,
    CodecResourceExhausted,
    InputFailed,
    OutputFailed,
    ReconfigureFailed,
    DeviceQuirk,
    Timeout,
    Fatal,
}
```

---

# 124. Failure Classification

Retryable:

```text
temporary resource
surface lost
```

Potentially non-retryable:

```text
codec unsupported
known broken profile
```

---

# 125. Codec Watchdog

If no output for expected time:

```text
detect stall
```

Then:

```text
flush
restart
fallback
```

---

# 126. Encoder Stall

Could be:

```text
camera stopped
surface detached
codec hung
```

Diagnostics distinguish.

---

# 127. Decoder Stall

Could be:

```text
no packets
jitter buffering
codec hung
surface lost
```

---

# 128. Diagnostics

Part 18 should expose:

```text
Video codec: AV1 hardware
Input: Surface
Output: Surface
Raw CPU copies: 0
Resolution: 1080p
FPS: 30
Bitrate: 2.5 Mbps
Decoder restarts: 0
```

---

# 129. Copy Counter

Maintain diagnostics counters:

```text
raw_frame_cpu_copy_count
raw_frame_cpu_copy_bytes
```

Production expectation for hardware path:

```text
0
```

or explain why not.

---

# 130. Hardware vs Software Metric

```text
encoder_backend
decoder_backend
```

---

# 131. Thermal Metric

Can correlate:

```text
codec choice
temperature
```

for tuning.

---

# 132. Frame Timing Metrics

Measure:

```text
camera→encoder
encoder latency
network
decoder latency
surface render
```

where observable.

---

# 133. End-to-End Latency

Use sender/receiver media timestamps carefully.

Clock synchronization limitations should be explicit.

---

# 134. Dropped Frames

Track:

```text
camera drop
encoder drop
network drop
late decoder drop
render drop
```

separately where possible.

---

# 135. Developer Overlay

Optional call diagnostics overlay:

```text
AV1 HW
1080p30
2.4 Mbps
RTT 42 ms
0.8% loss
Direct
```

---

# 136. User UI

Do not show codec internals by default.

Normal:

```text
HD
Connection good
```

---

# 137. Privacy

Diagnostics must not log:

```text
raw frame
screenshots
camera content
```

---

# 138. Security

Surface handles/pointers remain process-local.

Never serialize them.

---

# 139. Codec Input Validation

Encoded remote frames are untrusted.

Before decoder:

```text
frame size limits
codec state validation
rate limits
```

---

# 140. Decoder Attack Surface

Hardware decoder parses hostile media.

Use:

```text
OS security updates
bounded frame sizes
codec restart isolation
```

Do not pass malformed giant buffers blindly.

---

# 141. Software AV1 Fuzzing

Part 10 applies strongly to software codec glue/parsers.

---

# 142. Native API Fuzz Boundary

Do not fuzz Android codec driver directly with unbounded malformed data on production devices.

Use controlled test environments.

---

# 143. Resource Limits

Part 08:

```text
max encoded frame
max jitter bytes
max decoder instances
max encoder instances
max buffer pool
```

---

# 144. Call Priority

Audio codec and control frames get higher scheduler priority than video.

---

# 145. Bounded Codec Channels

Example:

```rust
tokio::sync::mpsc::channel(N)
```

with deliberate `N`.

No unbounded channels.

---

# 146. Worker Isolation

Codec thread failure should fail:

```text
media video subsystem
```

not messaging daemon.

---

# 147. Process Isolation

Android app normally hosts codec in same process.

If future reliability demands:

```text
media service process
```

could isolate codec crashes, but it adds IPC/Surface complexity.

Not required initially.

---

# 148. Dioxus Renderer Architecture

Suggested:

```text
VideoTile
  ↓
PlatformVideoSurface
  ↓
SurfaceId
  ↓
Rust MediaRuntime
  ↓
AndroidSurfaceDecoder
```

---

# 149. UI ↔ Media Commands

Only low-frequency commands:

```text
attach renderer
detach renderer
set visibility
set fit mode
```

No frames.

---

# 150. Renderer Registry

```rust
pub struct RendererRegistry {
    // RendererId → SurfaceHandle
}
```

Owned by Android media integration layer.

---

# 151. Surface Callback Contract

```rust
pub enum SurfaceEvent {
    Created { renderer: RendererId, surface: SurfaceHandle },
    Resized { renderer: RendererId, width: u32, height: u32 },
    Destroyed { renderer: RendererId },
}
```

---

# 152. Resize

Do not restart decoder for every layout size change.

Decoder resolution and display surface size are independent.

Use compositor scaling.

---

# 153. Resolution Negotiation

Sender resolution changes for network/battery reasons, not because UI tile changes by a few pixels.

---

# 154. High-DPI

Surface uses physical pixel dimensions.

Dioxus layout may use logical units.

Adapter converts correctly.

---

# 155. HDR / Color

Future support may require:

```text
color standard
transfer
range
bit depth
```

in codec negotiation.

---

# 156. Initial Color Scope

Start with:

```text
8-bit SDR
BT.709-like ordinary video profile
```

unless product specifically needs HDR.

---

# 157. Color Conversion

Avoid CPU conversion.

Use camera/codec/GPU path.

---

# 158. YUV Formats

Hardware surface path avoids choosing CPU YUV layout in most cases.

Software AV1 fallback will need explicit supported YUV formats.

---

# 159. Software YUV Contract

Example:

```rust
pub enum SoftwarePixelFormat {
    I420,
    Nv12,
}
```

Keep set small.

---

# 160. Conversion

If camera produces incompatible software format:

```text
use hardware/GPU conversion if possible
```

before CPU conversion.

---

# 161. Camera Capture API

The camera subsystem should provide:

```text
SurfaceTarget
```

for hardware mode.

Not:

```text
Vec<u8>
```

by default.

---

# 162. Hardware Pipeline Builder

```rust
let pipeline = AndroidHardwareVideoPipeline::builder()
    .codec(VideoCodec::Av1)
    .resolution(Resolution::HD1080)
    .fps(30)
    .build()?;
```

---

# 163. Encoder Surface

Pipeline exposes:

```text
camera target surface
```

through native handle abstraction.

---

# 164. Receiver Pipeline

```rust
let decoder = AndroidSurfaceDecoder::new(codec_config, renderer)?;
```

---

# 165. Session Integration

Call session owns:

```text
VideoSendSession
VideoReceiveSession
```

not raw codec objects directly.

---

# 166. Session State

```rust
pub enum VideoSessionState {
    Disabled,
    Starting,
    Active,
    Reconfiguring,
    Paused,
    Failed,
    Stopping,
}
```

---

# 167. Codec Swap

If negotiation changes:

```text
H.265 → H.264
```

perform coordinated:

```text
new codec config
decoder setup
keyframe
switch
```

---

# 168. Seamless Swap Goal

Avoid multi-second blank screen.

---

# 169. Surface Still Reused

Codec swap may reuse same output Surface.

---

# 170. Network Multipath

Part 12 can change path without touching Surface pipeline.

Media session remains logically identical.

---

# 171. Network Handoff

Example:

```text
Wi-Fi → cellular
```

Rust congestion controller adapts bitrate.

Codec may not restart.

---

# 172. Bluetooth

Bluetooth is not intended for full video streaming.

Routing should generally avoid video over BLE.

---

# 173. LAN

LAN direct can support high-quality video with low latency.

---

# 174. Relay

Relay path may require lower bitrate based on congestion.

---

# 175. DTN

Realtime video does not use DTN.

Recorded clips/files do.

---

# 176. Background File Video

A sent video attachment uses file subsystem, not live Surface path.

---

# 177. Video Thumbnail

Can be generated separately.

Do not decode entire video into CPU frames for every UI display.

---

# 178. Preview Frame

One-time decoded frame is acceptable for thumbnail.

Separate from call renderer.

---

# 179. Build Feature

Example:

```toml
android-hw-video = []
av1-software = []
```

---

# 180. Platform Gate

`comm-media-android` builds only for:

```text
target_os = "android"
```

---

# 181. Mock Backend

Desktop/unit tests need:

```text
MockSurfaceCodec
```

for state-machine testing.

---

# 182. Android Instrumentation Tests

Required for actual:

```text
MediaCodec
Surface
camera
```

behavior.

---

# 183. Device Matrix

Test:

```text
low-end Android
mid-range Android
recent flagship
AV1 hardware device
H.265-only device
H.264 fallback device
```

---

# 184. Android Version Matrix

Support must be based on chosen minSdk/targetSdk.

Keep exact API gating in platform adapter.

---

# 185. Codec Capability Test

For every test device:

```text
enumerate advertised codecs
start encoder
encode test frames
decode
render
```

---

# 186. Surface Lifecycle Test

Repeated:

```text
create
destroy
recreate
rotate
background
foreground
```

during live call.

Expected:

```text
no crash
no dangling surface
recovery
```

---

# 187. Camera Switch Test

Switch front/rear repeatedly.

---

# 188. Network Handoff Test

While video active:

```text
Wi-Fi → cellular → Wi-Fi
```

Surface remains stable.

---

# 189. Encoder Restart Test

Force hardware codec error.

Expected:

```text
bounded restart
keyframe
resume
```

---

# 190. Decoder Restart Test

Same.

---

# 191. Hardware Resource Exhaustion Test

Open competing codec instances.

Expected:

```text
typed resource failure
quality degradation
```

---

# 192. Zero-Copy Acceptance Test

For hardware path, instrumentation should verify app does not route raw frame-sized ByteArrays/Vecs through Kotlin/Rust bridge.

---

# 193. Memory Bandwidth Benchmark

Compare:

```text
legacy CPU-copy pipeline
vs
Surface pipeline
```

Measure:

```text
CPU
battery
temperature
memory copies
frame latency
```

---

# 194. Battery Benchmark

Run 30–60 minute video call.

Track:

```text
battery drain
thermal
dropped frames
```

---

# 195. 1080p Test

Target stable:

```text
1080p30
```

on suitable device.

---

# 196. 4K

Only enable if:

```text
device capability
network
battery
UI need
```

support it.

4K should not be default video-call mode.

---

# 197. Low-End Device

Gracefully use:

```text
720p
480p
```

rather than saturating device.

---

# 198. Software AV1 Benchmark

Measure whether fallback is actually viable on target hardware.

If not:

```text
audio-only / no video
```

is better than unusable thermal overload.

---

# 199. Crash Recovery

Live calls are ephemeral.

After app process death:

```text
call ends/reconnects according to call protocol
```

No attempt to restore stale Surface object.

---

# 200. Surface Persistence

Never persist:

```text
SurfaceId
ANativeWindow pointer
codec instance
```

across process restart.

---

# 201. Diagnostics Persistence

Only aggregate failure counters may persist.

---

# 202. Security Invariants

1. Raw video frames do not normally cross Kotlin ↔ Rust as byte arrays.
2. Hardware encoder surface is the camera target in hardware mode.
3. Hardware decoder renders directly to output Surface.
4. Surface pointers never outlive acquired references.
5. UI destruction invalidates/rebinds renderer safely.
6. Codec queues are bounded.
7. Network backpressure cannot create unbounded live-video latency.
8. Hardware capability is detected at runtime.
9. Software fallback is AV1 only, according to product requirement.
10. Hardware codec failure cannot crash messaging core.
11. Video priority never starves audio/control.
12. Surface handles are process-local and never serialized.
13. Raw frames are never written to logs.
14. Emergency/battery/thermal policy can downgrade or disable video.
15. Dioxus deals with renderer handles, not frame byte buffers.

---

# 203. Suggested Workspace Integration

```text
crates/
├── comm-media-core/
├── comm-media-transport/
├── comm-media-av1/
├── comm-media-android/
├── comm-media-sync/
├── comm-media-diagnostics/
└── comm-ui-video/

platform/
└── android/
    ├── media/
    ├── camera/
    └── surface/
```

---

# 204. `comm-media-core`

Owns:

```text
codec-independent session
timestamp
bitrate policy
quality ladder
```

---

# 205. `comm-media-transport`

Owns:

```text
packetization
reassembly
jitter
keyframe feedback
```

---

# 206. `comm-media-av1`

Owns:

```text
software AV1 path
```

No Android-specific code.

---

# 207. `comm-media-android`

Owns hardware codec/surface integration.

---

# 208. `comm-ui-video`

Owns Dioxus view model and renderer registration.

No codec dependency.

---

# 209. Production Rollout Phases

## Phase 1 — Native Surface Wrapper

```text
ANativeWindow
RAII
surface lifecycle
```

## Phase 2 — Hardware Decode to Surface

```text
H.264
H.265
AV1 where available
```

## Phase 3 — Hardware Encode from Surface

```text
camera → encoder
```

## Phase 4 — Dioxus Renderer Bridge

```text
renderer ID
surface create/destroy
```

## Phase 5 — Adaptive Media

```text
bitrate
fps
resolution
keyframe
```

## Phase 6 — Software AV1 Fallback

```text
native raw-buffer access
no Kotlin ByteArray hot path
```

## Phase 7 — Device Quirks

```text
capability probe
fallback database
```

## Phase 8 — Hardening

```text
lifecycle
network handoff
codec stalls
thermal
battery
soak
```

---

# 210. Definition of Done

Part 25 is complete when:

- Android video calls can run with hardware encoder Surface input
- hardware decoder can render directly to a Surface
- normal hardware media path does not copy raw frame ByteArrays between Kotlin and Rust
- Dioxus video components operate on renderer/surface handles rather than raw frames
- H.264, H.265, and AV1 hardware capability is detected dynamically
- AV1 remains the only software video codec fallback
- camera preview can remain surface-based
- codec instances are bounded and resource-aware
- live video queues cannot grow without limit
- bitrate/FPS/resolution adaptation works without unnecessary codec restarts
- decoder recovery/keyframe-request behavior is defined
- Surface destruction/recreation is safe during rotation/background/navigation
- hardware failure degrades to another valid codec/backend or audio-only
- battery/thermal policy can reduce or disable video
- diagnostics can distinguish hardware/software path and count raw-frame copies
- real Android device tests demonstrate the Surface path is materially more CPU/battery efficient than the copied-frame path

---

# 211. Relationship to Previous Parts

This part strengthens:

```text
03 — Transport / Routing
07 — Capability Negotiation
08 — Resource Limits
10 — Fuzzing / Protocol Tests
12 — Multipath
13 — Battery-Aware Scheduling
16 — Daemon / Runtime
17 — Emergency Priority
18 — Diagnostics
19 — C ABI / FFI
```

and connects directly to the previously defined:

```text
Android H.264 hardware codec
Android H.265 hardware codec
Android AV1 hardware codec
AV1 software codec
Dioxus mobile UI
```

---

# 212. Final Architecture

## Send

```text
                    ANDROID CAMERA
                          │
                          ▼
                   BufferQueue/Surface
                    ┌─────┴──────┐
                    │            │
               Local Preview   Encoder Surface
                    │            │
                    │       Hardware Codec
                    │            │
                    │      Encoded Access Unit
                    │            │
                    └────────────┼─────────────┐
                                 ▼             │
                           Rust Packetizer     │
                                 │             │
                              Iroh/QUIC        │
```

## Receive

```text
Iroh/QUIC
    │
    ▼
Rust Depacketizer
    │
    ▼
Encoded Access Unit
    │
    ▼
Android Hardware Decoder
    │
    ▼
Output Surface / ANativeWindow
    │
    ▼
Android Compositor
    │
    ▼
Dioxus Video View
```

Normal receive path contains:

```text
zero raw-frame Kotlin ↔ Rust copies
```

---

# 213. Final Principle

The Rust application should control the video system without becoming the transport mechanism for every pixel.

The efficient Android architecture is:

```text
Rust controls
Android hardware moves pixels
Rust moves compressed media
```

not:

```text
Android creates pixels
Kotlin copies pixels
Rust copies pixels
codec copies pixels
UI copies pixels
```

That distinction becomes increasingly important at:

```text
1080p
60 fps
4K
long calls
battery-powered devices
```

Part 25 therefore makes the Android media path genuinely production-grade rather than merely functionally correct.
