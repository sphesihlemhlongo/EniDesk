# ADR 0001: Initial Architecture Selection

## Status
Accepted

## Context
We are building a free, open-source cross-platform remote desktop application (EniDesk) capable of low-latency screen capture and input injection. It needs to work securely over the internet across strict NATs and firewalls.

## Decision
We will use the following technology stack:
- **Language:** Rust (for core/daemon) because of memory safety and zero-cost abstractions needed for high-performance screen capture and system input hooks.
- **Frontend:** Tauri + React/TypeScript, providing a cross-platform lightweight webview UI without the overhead of Electron.
- **Networking:** WebRTC for P2P encrypted low-latency streaming.
- **Signaling Server:** Custom Rust WebSocket server + STUN/TURN for NAT traversal.
- **Screen Capture:** Rust `xcap` crate for cross-platform frame grabbing.
- **Input Injection:** Rust `enigo` crate for cross-platform mouse/keyboard control.
- **Communication:** WebRTC DataChannels for both control and screen streaming (JPEG/Base64).

## Consequences
- **Positive:** Very low memory footprint and high performance. Cross-platform support through Tauri and standard Rust crates.
- **Negative:** Rust learning curve and potentially complex WebRTC Rust bindings compared to JavaScript or Go.
