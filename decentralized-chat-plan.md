# Implementation Plan: Decentralized Mobile Chat App (Reticulum + Tauri V2 + Astro + Honker)

## 1. Objective
Build a serverless, peer-to-peer mobile chat application that utilizes the Reticulum Network Stack for encrypted communication, **SQLite + webhooks** for local-first storage, transactional pub/sub, media (AVIF/Opus) and location pings, and Astro + Tailwind 4 for a lightweight, high-performance UI.

## 2. Tech Stack Refinement
- **Networking:** `reticulum-rs` for E2EE mesh networking and `AutoInterface` for local discovery.
- **Frontend:** Astro (SSG) + Tailwind CSS 4 + Svelte Runes (for reactive UI state).
- **Database:** `libsql` (Rust-level, bundled) for local-first serverless storage.
- **Pub/Sub & Queuing:** webhook for durable events and internal notifications.
- **Media Processing:** 
    - **Images:** `image` crate (AVIF conversion).
    - **Audio:** `audiopus` crate (Opus encoding).
- **Mobile Bridge:** Tauri V2 with Mobile Plugins (Geolocation, Notifications, Persistence).

## 3. Key Architectural Decisions

### A. Peer Discovery & Identity
- **Local Discovery:** Enable Reticulum `AutoInterface` by default for zero-config WiFi/Ethernet mesh discovery.
- **Manual Discovery:** Use Reticulum's Identity hashes (truncated) as user IDs. Users can share these IDs/QR codes.
- **Search:** Implement a local "Contacts" table in libSQL. When a user manually adds an ID, the app attempts to "announce" or "ping" that identity via Reticulum to establish a link.
- **Auth:** Avoid external providers like Clerk to maintain 100% serverless/offline integrity. Identity is derived directly from Reticulum's cryptographic keys.

### B. Data Flow
1. **Outgoing:** UI (Astro) -> Tauri Command (Rust) -> Media Processing (AVIF/Opus) -> libSQL (Save) -> Reticulum (Broadcast).
2. **Incoming:** Reticulum (Receive) -> Rust Listener -> libSQL (Save) -> Tauri Event -> UI (Update).

## 4. Implementation Steps

### Phase 1: Foundation (Environment & Scaffold)
- [ ] Initialize Tauri V2 project with Astro frontend.
- [ ] Configure `astro.config.mjs` for Static Site Generation (SSG).
- [ ] Integrate Tailwind CSS 4 using `@tailwindcss/vite`.
- [ ] Setup `src-tauri` with cross-compilation targets for Android and iOS.

### Phase 2: Core Rust Services (Networking & DB)
- [ ] Integrate `reticulum-rs` into the Tauri backend.
- [ ] Configure `AutoInterface` and basic Identity management.
- [ ] Setup `libsql` with a local file-based schema (Messages, Contacts, Blobs).
- [ ] Implement Rust-level "Services" for AVIF image conversion and Opus audio encoding.

### Phase 3: Frontend & Tauri Bridge
- [ ] Create Tauri Commands for: `send_message`, `get_history`, `add_contact`, `update_location`.
- [ ] Implement Svelte Runes for the chat interface to handle real-time message updates.
- [ ] Integrate Tauri Geolocation plugin for location pings.
- [ ] Add the project logo (`static/app.ico`) to the app bundle and UI.

### Phase 4: Mobile Optimization & Verification
- [ ] Implement background task persistence for Reticulum on Android/iOS.
- [ ] Test peer discovery between two mobile devices on the same mesh network.
- [ ] Verify image/audio compression and storage in libSQL.
- [ ] Final UI polish with Tailwind 4.

## 5. Verification & Testing
- **Local Mesh Test:** Verify two devices find each other via `AutoInterface` without internet.
- **Media Integrity:** Confirm AVIF and Opus files are correctly stored in libSQL and rendered in the UI.
- **Persistence:** Ensure messages remain available after app restart and device reboot.
- **E2EE:** Verify that only the intended Reticulum Identity can decrypt received messages.
