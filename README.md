# EniDesk

EniDesk is a free, open-source, and secure cross-platform remote desktop application built with Rust and Tauri. It allows users to control different computers remotely with low latency, featuring an AnyDesk-like 9-digit ID system and peer-to-peer (P2P) encryption via WebRTC.

## 🚀 Features

- **Cross-Platform:** Works on Windows, macOS, and Linux.
- **Low Latency:** Utilizes WebRTC for high-performance screen streaming and input injection.
- **Secure:** End-to-end encrypted P2P connections; passwords never leave your devices.
- **Lightweight:** Built with Tauri, ensuring a small binary size and minimal RAM usage.
- **Unattended Access:** Set a permanent password for remote access to your own devices.

## 🛠 Tech Stack

- **Backend (Client):** Rust (Tauri, xcap, enigo)
- **Frontend (Client):** React, TypeScript, Vite
- **Signaling Server:** Rust (Axum, WebSockets)
- **Networking:** WebRTC (DataChannels for control and streaming)

## 📦 Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- OS-specific Tauri dependencies (see [Tauri Docs](https://tauri.app/v1/guides/getting-started/prerequisites))

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/EniDesk.git
   cd EniDesk
   ```

2. **Start the Signaling Server:**
   ```bash
   cd server
   cargo run
   ```

3. **Start the Client (Development Mode):**
   ```bash
   cd client
   npm install
   npm run tauri dev
   ```

## 🏗 Build for Production

To generate installable binaries (MSI, DMG, AppImage):
```bash
cd client
npm run tauri build
```
The output will be in `client/src-tauri/target/release/bundle/`.

## 📄 License
This project is open-source and available under the MIT License.
