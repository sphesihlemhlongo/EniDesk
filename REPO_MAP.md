# Repository Map: EniDesk

This document provides a structural overview of the EniDesk repository to help developers and AI agents navigate the codebase.

## 📂 Project Structure

```text
EniDesk/
├── Cargo.toml                # Root workspace configuration
├── README.md                 # Project overview and setup instructions
├── REPO_MAP.md               # This file
├── .gitignore                # Global ignore patterns
├── docs/                     # Documentation and Design Records
│   └── adr/                  # Architecture Decision Records (ADRs 0001-0004)
├── server/                   # Signaling Server (Rust/Axum)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs           # WebSocket routing and signal relay logic
└── client/                   # Desktop Client (Tauri/React)
    ├── package.json          # Node.js dependencies
    ├── src/                  # Frontend (React/TypeScript)
    │   ├── App.tsx           # Main UI & WebRTC P2P/Signaling logic
    │   └── App.css           # Styling
    └── src-tauri/            # Backend (Rust)
        ├── Cargo.toml
        ├── tauri.conf.json   # Tauri configuration
        └── src/
            ├── main.rs       # Entry point
            └── lib.rs        # Screen capture & input injection commands
```

## 🏗 Key Architectures

### 1. Signaling Flow
- Located in `server/src/main.rs`.
- Uses WebSockets to facilitate the exchange of WebRTC SDP Offers and Answers between peers identified by random 9-digit IDs.

### 2. P2P Connectivity
- Located in `client/src/App.tsx`.
- Uses the browser's native `RTCPeerConnection` within the Tauri webview.
- Establishes a `RTCDataChannel` named `enidesk-control` for all communication (Video frames + Input events).

### 3. Screen Capture & Input Injection
- Located in `client/src-tauri/src/lib.rs`.
- **Capture:** Uses `xcap` to grab frames, `image` (JPEG) for compression, and emits them to the frontend as Base64.
- **Injection:** Uses `enigo` to simulate mouse/keyboard events received from the remote peer.

### 4. Security
- Located in both `App.tsx` and `lib.rs`.
- Implements a P2P password challenge over the encrypted DataChannel.

## 🛠 Contribution Guidelines
- **Rust:** Follow idiomatic patterns and document functions using `///`.
- **React:** Use functional components and hooks.
- **Decisions:** Major architectural changes must be documented with a new ADR in `docs/adr/`.
