# ADR 0003: Video Streaming over DataChannel

## Status
Accepted

## Context
We need to send screen frames from the host to the viewer. While WebRTC `MediaStream` (video tracks) is the standard for video, it is complex to implement in a hybrid Rust/Tauri environment without using a full browser-based capture or complex GStreamer/ffmpeg pipelines in Rust.

## Decision
For the MVP, we are capturing screen frames in Rust, encoding them as JPEG, and sending them as Base64 strings over a WebRTC `RTCDataChannel`. 

## Consequences
- **Positive:** Extremely simple to implement and requires no complex media pipelines. Works within the existing DataChannel architecture.
- **Negative:**
    - **Performance:** Base64 encoding/decoding adds overhead.
    - **MTU Limits:** Large JPEG frames might exceed `RTCDataChannel` message size limits (typically 64KB-256KB depending on browser). This may lead to dropped frames or connection instability if frames are too large.
    - **Latency:** JPEG encoding at 10 FPS is not as efficient as H.264/VP8 hardware encoding.

## Mitigation
If we encounter message size issues, we will implement a simple chunking mechanism or switch to a more robust video track implementation in a future phase.
