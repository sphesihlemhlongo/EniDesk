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

## 🎮 How to Use EniDesk

To test the remote control capabilities, you need the app running on two machines (or two windows on the same machine for local testing):

### On the Host Computer (The PC being controlled):
1. Open EniDesk.
2. Note your 9-digit **Your Address** (e.g., `123456789`).
3. Enter a secure password in the **Set Access Password** field.
4. Wait for the Viewer to connect. Once the status says "P2P Connected", click **Start Sharing Screen**.

### On the Viewer Computer (The PC controlling the host):
1. Open EniDesk.
2. Under **Remote Desk**, enter the Host's 9-digit address.
3. Click **Connect**.
4. You will be prompted for an **Authentication Required** password. Enter the password the Host set.
5. The remote screen will appear. You can now move your mouse, click, and type to control the remote computer!

## 🏗 Build for Production

### Automated CI/CD (GitHub Actions)
This repository includes a GitHub Actions workflow (`.github/workflows/release.yml`) that automatically builds the application for **Windows, macOS, and Linux**. 
Whenever you push to the `main` branch, GitHub will spin up virtual machines, compile the app, and generate a draft Release containing the `.msi`, `.exe`, `.dmg`, `.app`, `.AppImage`, and `.deb` installers.

### Manual Build
To manually generate installable binaries for your current operating system:
```bash
cd client
npm run tauri build
```
The output will be in `client/src-tauri/target/release/bundle/`.

## 📄 License
This project is open-source and available under the MIT License.
