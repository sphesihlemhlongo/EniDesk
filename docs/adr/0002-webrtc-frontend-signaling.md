# ADR 0002: WebRTC Signaling in the Frontend

## Status
Accepted

## Context
Phase 2 requires establishing a WebRTC P2P connection between the host and viewer clients. This involves exchanging SDP Offers, Answers, and ICE candidates via the signaling server. We need to decide whether this logic should reside in the Rust backend or the React/TypeScript frontend.

## Decision
We will implement the initial WebRTC connection and DataChannels in the React/TypeScript frontend using the browser's native `RTCPeerConnection` API. 

## Consequences
- **Positive:** Reduces the complexity of Rust C++ bindings (`webrtc-rs` or `libwebrtc`) and speeds up development. It takes advantage of Tauri's built-in modern webview capabilities.
- **Negative:** If high-performance video encoding requires low-level access to WebRTC's C++ packet pipelines, passing video frames from Rust to the frontend webview via IPC might introduce latency. We will evaluate performance and migrate video streams to Rust if necessary, while keeping signaling and DataChannels in the frontend for control data.
