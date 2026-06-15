

# Multi-Room Asynchronous WebSocket Chat Server From Scratch

A high-performance, asynchronous multi-room chat server built from scratch in **Rust** using **Tokio**, without relying on high-level protocol abstraction crates like `tungstenite`, `axum`, or `warp`. 

This project implements the full **RFC 6455 WebSocket Protocol** specification manually, handling raw TCP streams, binary framing structures, bitwise bitmask parsing, cryptographic handshakes, and multi-threaded atomic state management.


## 🚀 Key Architectural Features

* **Custom HTTP Protocol Upgrade Handshake:** Manually extracts client security tokens (`Sec-WebSocket-Key`), performs cryptographic concatenation using the global Magic GUID, and computes SHA-1 hashes and Base64 signatures to authorize connections.
* **Low-Level Binary Frame Dissection:** Decodes incoming payloads using custom bitwise operators (`& 0x0F` to extract Opcodes, `& 0x7F` to isolate payload length indicators).
* **Cyclic XOR Bitwise Decryption:** Implements the official WebSocket client masking specification to dynamically parse and unmask encrypted browser payloads.
* **Thread-Safe Multi-Room State Engine:** Utilizes cross-thread atomic reference counting pointers (`Arc<Mutex<ChatAppState>>`) to safe-guard memory mapping pathways across isolated background runtime tasks.
* **Asynchronous Message Multiplexing:** Leverages Tokio's powerful `tokio::select!` macro pattern to concurrently race network socket events (`socket.read`) side-by-side with real-time in-memory room broadcasts (`rx.recv`).

---

## 📐 System Protocol Data Flow

```text
[Main Concurrent TCP Listener Loop] 
         │
         ▼ (Client attempts handshake on Port 8080)
  listener.accept() ───► Duplicates Global Directory Pointer (Arc::clone)
         │
         ▼ (Asynchronous Task Isolation Hand-off)
  tokio::spawn(async move { handle_connection(...) })
         │
   ┌─────┴────────────────────────────────────────────────────────┐
   │  [handle_connection Background Worker Task Pipeline]        │
   │                                                              │
   │  Step 1: The Strict HTTP Upgrade Handshake                   │
   │  • Reads raw HTTP header string sequence from TcpStream.     │
   │  • Resolves cryptographic math: Base64(Sha1(Key + GUID)).     │
   │  • Writes raw 101 Switching Protocols response bytes.       │
   │                                                              │
   │  Step 2: State Room Entry Routing & Directory Registration   │
   │  • Awaits the FIRST WebSocket message payload.               │
   │  • Validates command structure: "JOIN:room_name:username".   │
   │  • Seizes thread Mutex guard & registers user profile.       │
   │  • Spawns or subscribes to tokio::sync::broadcast tower.     │
   │                                                              │
   │  Step 3: Permanent Multiplexed Channel/Socket Polling        │
   │  • Enters infinite tokio::select! execution race block.      │
   │  • Event A: Decodes browser frame via XOR bitwise masks,     │
   │             and broadcasts text out to the channel tower.    │
   │  • Event B: Catches tower transmissions and packs them into  │
   │             unmasked text server frames (0x81) to client.    │
   └──────────────────────────────────────────────────────────────┘

```

---

## 🛠️ Project Structure

```text
├── Cargo.toml          # Project configuration and dependency manifests (sha1, base64, tokio)
└── src
    ├── main.rs         # Async networking engine, protocol parsing loops, handshake architecture
    └── state.rs        # In-memory structural mapping layout definitions for state and users

```

---

## 💻 Tech Stack & Primitives

* **Language:** Rust (Stable)
* **Asynchronous Core:** `tokio` (with `full` macros & runtime configurations)
* **Concurrency Models:** `tokio::sync::broadcast` channels, `Arc` (Atomic Reference Counting), `Mutex` (Mutual Exclusion Lock), `tokio::select!`
* **Cryptographic Digests:** `sha1`, `base64`

---

## 🏃 Getting Started & Local Testing

### 1. Run the Server

Clone the repository and spin up the server container using cargo:

```bash
cargo run

```

The terminal will display: `Server is live and listening on 127.0.0.1:8080`.

### 2. Live Browser Orchestration Testing

Open any modern web browser (e.g., Chrome, Brave, Firefox), navigate to a standard public page (like `https://example.com`), launch the Developer Console (`F12`), and execute the following scripts across **two separate tabs** to test real-time broadcasting:

**Tab 1 (User: Sathwik):**

```javascript
const ws = new WebSocket("ws://127.0.0.1:8080");
ws.onmessage = (e) => console.log("Tab 1 heard:", e.data);

// Execute once connection shifts to OPEN
ws.send("JOIN:rust-room:Sathwik");
ws.send("Hello from the systems side!");

```

**Tab 2 (User: Guest):**

```javascript
const ws = new WebSocket("ws://127.0.0.1:8080");
ws.onmessage = (e) => console.log("Tab 2 heard:", e.data);

// Execute once connection shifts to OPEN
ws.send("JOIN:rust-room:Guest");

```

When Tab 1 broadcasts text data, Tab 2 will instantaneously print out the unmasked server packet payload via its active asynchronous pipeline.

