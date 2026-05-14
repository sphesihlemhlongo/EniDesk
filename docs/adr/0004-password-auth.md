# ADR 0004: Password Authentication Flow

## Status
Accepted

## Context
We need to secure remote access to prevent unauthorized users from controlling a computer just by knowing its 9-digit ID.

## Decision
We implemented a Peer-to-Peer password verification flow:
1.  **Host** sets a password in the UI, which is stored in the Rust backend state.
2.  **Viewer** establishes a WebRTC DataChannel.
3.  Upon connection, the **Viewer** is prompted for a password.
4.  The password is sent over the encrypted DataChannel as an `auth-request`.
5.  The **Host** Rust backend verifies the password and sends an `auth-response`.
6.  The **Viewer** only enables input injection and screen viewing if the response is successful.

## Consequences
- **Positive:** Simple, effective unattended access security. Password is never sent to the signaling server; it only travels over the E2E encrypted WebRTC channel.
- **Negative:** Basic string comparison is used. In the future, we should use a proper KDF (Key Derivation Function) like Argon2 to hash the password before storage and comparison.
