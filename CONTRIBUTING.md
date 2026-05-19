# Contributing to EniDesk

Thank you for your interest in contributing to EniDesk! As an open-source project, we welcome contributions in the form of bug reports, feature requests, documentation, and code.

## 🤝 How to Contribute

### 1. Reporting Bugs
- Use the GitHub Issue tracker to report bugs.
- Include steps to reproduce, expected behavior, and screenshots if applicable.

### 2. Suggesting Enhancements
- Open a "Feature Request" issue.
- Describe the use case and how it benefits the users.

### 3. Submitting Code
- **Fork** the repository and create your branch from `main`.
- **Implement** your changes, following the existing code style.
- **Test** your changes thoroughly.
- **Document** any new features or architectural changes in `docs/adr/`.
- **Submit a Pull Request** with a clear description of the changes.

## 🛠 Development Workflow

1.  **Environment Setup:** Follow the "Prerequisites" section in the [README.md](./README.md).
2.  **Signaling Server:** Run the signaling server locally while developing:
    ```bash
    cd server && cargo run
    ```
3.  **Local P2P Testing:** You can run two instances of the client locally to test the connection. Use two different terminal windows:
    ```bash
    cd client && npm run tauri dev
    ```

## 📜 Coding Standards

- **Rust:**
    - Use `cargo fmt` before committing.
    - Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
    - Document public functions with `///`.
- **TypeScript / React:**
    - Use functional components and hooks.
    - Prefer explicit typing over `any`.

## 🏗 Architecture Decisions (ADR)
We use ADRs to track major engineering decisions. If your contribution changes the way the app works (e.g., switching from JPEG to H.264), please add a new ADR in `docs/adr/`.

## 🛡 Security
If you find a security vulnerability, please do not open a public issue. Instead, contact the maintainers directly.
