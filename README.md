# RNS

A decentralized peer-to-peer mobile chat application built on the **Reticulum Network Stack** and **Yggdrasil** mesh networking. RNS provides end-to-end encrypted communication, local-first data storage, and true offline capability.

## Overview

RNS is a serverless, privacy-first chat application that combines a Rust backend with Svelte frontend. It leverages the Reticulum Network Stack for encrypted routing, Yggdrasil for global mesh connectivity, and local SQLite storage for complete data autonomy. The app works on Android, iOS, and desktop platforms via Tauri V2.

**Key Features:**
- 🔐 **End-to-End Encrypted** - Uses Reticulum Network Stack cryptography
- 📱 **Mobile-First** - Built with Tauri V2 for Android, iOS, and desktop
- 🌐 **Fully Decentralized** - No servers required, peer-to-peer communication
- 🗺️ **Mesh Networking** - Works over Yggdrasil IPv6 mesh and local WiFi
- 💾 **Local-First** - SQLite storage keeps your data private and persistent
- 📸 **Rich Media** - Send AVIF-compressed images and Opus-encoded audio
- 📍 **Location Sharing** - Optional location pings over the mesh network

## Tech Stack

### Networking
- **Reticulum Network Stack** (`reticulum-rs`) - Application-agnostic encryption and routing
- **Yggdrasil** - End-to-end encrypted IPv6 mesh networking
- **AutoInterface** - Zero-config local peer discovery over WiFi/Ethernet

### Frontend
- **Astro** 6.4.8 - Static Site Generation (SSG) framework
- **Svelte** 5.56.3 - Reactive UI components with Svelte Runes
- **Tailwind CSS** 4.3.1 - Utility-first styling

### Backend (Rust)
- **Tauri** 2.11.2 - Desktop and mobile bridge
- **rusqlite** 0.31.0 - Local SQLite database
- **tokio** - Async runtime
- **image** 0.25 - AVIF image compression
- **audiopus** 0.3.0 - Opus audio encoding
- **ed25519-dalek** - Cryptographic signatures

### Platform Support
- **Android** - via Tauri with geolocation and barcode scanner
- **iOS** - via Tauri with geolocation
- **Desktop** (Linux/macOS/Windows) - via Tauri

## Architecture

### Data Flow

**Outgoing Messages:**
```
UI (Svelte) → Tauri Command (Rust) → Media Processing (AVIF/Opus) 
→ SQLite (Local Storage) → Reticulum Network Stack (Encrypted Broadcast)
```

**Incoming Messages:**
```
Reticulum Network Stack (Receive) → Rust Listener → SQLite (Store) 
→ Tauri Event → UI (Real-time Update)
```

### Peer Discovery & Identity

- **Local Discovery**: Automatic peer detection via Reticulum AutoInterface on local WiFi/Ethernet
- **Manual Discovery**: Share Reticulum Identity hashes (display as QR codes) with other users
- **Global Mesh**: Connect over Yggdrasil for global peer-to-peer communication
- **Identity**: Cryptographically derived from Reticulum keys (100% serverless, no external auth needed)

## Getting Started

### Prerequisites

- **Rust** 1.70+ (for building)
- **Node.js** 22.12.0+ (for frontend)
- **Tauri CLI** (for building desktop/mobile apps)
- **Android NDK** (optional, for Android builds)
- **Xcode** (optional, for iOS builds)

### Installation

```bash
# Clone the repository
git clone https://github.com/lfakh/rns.git
cd rns

# Install dependencies
npm install

# Build the Rust backend
cd src-tauri && cargo build && cd ..
```

### Development

```bash
# Run development server with Tauri
npm run tauri dev

# Build for production
npm run tauri build

# Preview static site
npm run preview
```

### Building for Mobile

#### Android
```bash
# Via GitHub Actions (recommended)
# Push to repository and use .github/workflows/android-build.yml
# Download the APK from the Actions artifacts

# Or locally with Android NDK set up
npm run tauri android build
```

#### iOS
```bash
# Requires Xcode and iOS dev setup
npm run tauri ios build
```

## Project Structure

```
rns/
├── src/                          # Astro frontend (SSG)
│   ├── pages/                    # Astro pages
│   ├── components/               # Svelte components
│   └── layouts/                  # Page layouts
├── src-tauri/                    # Tauri Rust backend
│   ├── src/
│   │   ├── lib.rs               # Main library entry
│   │   └── commands/            # Tauri commands
│   └── Cargo.toml               # Rust dependencies
├── public/                       # Static assets
├── package.json                 # Node dependencies
├── Cargo.toml                   # Root Rust config
├── astro.config.mjs             # Astro configuration
├── svelte.config.js             # Svelte configuration
├── tsconfig.json                # TypeScript configuration
├── decentralized-chat-plan.md   # Architecture & design docs
├── plan-progress.md             # Implementation progress
└── LICENSE                      # Apache 2.0 License
```

## How to Test

### Testing on Desktop

```bash
npm run tauri dev
```

This launches the desktop app with hot-reload enabled.

### Testing on Android (GitHub)

1. **Commit and push** your changes to GitHub
2. **Go to Actions** → "Build Android APK" workflow
3. **Download the APK** from the job artifacts
4. **Install on two devices** and test peer discovery
5. **Verify communication** over local WiFi or Yggdrasil mesh

### Testing Peer Discovery

1. Launch the app on two devices
2. Note the **Reticulum Identity** displayed at the top
3. Manually add the other device's identity in the Contacts
4. Send a test message - it should arrive encrypted via Reticulum
5. Try sharing your location for geolocation testing

## Current Implementation Status

### ✅ Completed
- Tauri V2 project with Astro + Svelte frontend
- Reticulum Network Stack integration
- Yggdrasil mesh networking
- SQLite local database with message/contact/blob storage
- Tauri commands for messaging, contacts, location, and media
- AVIF image compression and Opus audio encoding
- Geolocation plugin for location pings
- Android/iOS cross-compilation setup

### 🚧 In Progress
- Background task persistence (Android/iOS)
- UI polish and responsive design refinement

### 📋 Upcoming
- **OrbitDB Integration** - Distributed database for truly decentralized data sync
- **Web3 User Discovery** - IPFS/DHT-based user lookup system
- **Enhanced Yggdrasil Integration** - Automatic network status UI and advanced peer management
- **Encryption Settings UI** - User controls for E2EE verification
- **Message Reactions** - Emoji and rich message interactions

## Contributing

Contributions are welcome! We're actively seeking help with several key features:

### High-Priority Features for Contributors

#### 1. **OrbitDB Integration for Distributed Database**
   - Implement OrbitDB as a peer-to-peer distributed database layer
   - Enable message sync across multiple devices without centralized storage
   - Create Rust bindings or WASM integration with OrbitDB
   - **Impact**: Provides a fully decentralized data persistence layer that complements Reticulum routing
   - **Difficulty**: Medium to Hard

#### 2. **Web3 User Discovery System**
   - Build a decentralized user discovery mechanism leveraging web3 protocols
   - Integrate with IPFS/DHT for peer discovery
   - Create identity management tied to cryptographic keys
   - Implement privacy-preserving user lookup without exposing network topology
   - **Impact**: Enables users to find and connect with each other in a fully decentralized manner
   - **Difficulty**: Hard

#### 3. **Yggdrasil Network Integration (Recommended Starting Point)**
   - Enhance Yggdrasil integration with network status monitoring
   - Build UI to display mesh peer connections and network health
   - Implement automatic fallback between local WiFi and Yggdrasil mesh
   - Improve peer coordinate discovery and visualization
   - **Why start here?**: Yggdrasil and Reticulum work together seamlessly for end-to-end encrypted routing
   - **Difficulty**: Easy to Medium

### Getting Started

1. **Pick an issue** from the list above or check [open issues](https://github.com/lfakh/rns/issues)
2. **Fork the repository** and create a feature branch
3. **Review the architecture docs** in `decentralized-chat-plan.md` and `plan-progress.md`
4. **Implement your changes** following Rust and Svelte best practices
5. **Test thoroughly** (especially on mobile if applicable)
6. **Submit a Pull Request** with a clear description of your changes

### Development Guidelines

- Follow Rust naming conventions (snake_case for functions/variables)
- Use descriptive commit messages
- Write tests for new functionality
- Update documentation as needed
- Ensure code compiles without warnings
- Test changes on both desktop and mobile if possible

## Architecture Documents

- **[decentralized-chat-plan.md](decentralized-chat-plan.md)** - Detailed architecture, tech stack rationale, and implementation phases
- **[plan-progress.md](plan-progress.md)** - Current implementation progress, testing guide for Android builds

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Community & Support

- **Report Issues**: [GitHub Issues](https://github.com/lfakh/rns/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lfakh/rns/discussions)
- **Contributing**: See [CONTRIBUTING.md](CONTRIBUTING.md) (coming soon)

## Disclaimer

RNS is a peer-to-peer application with end-to-end encryption. While we aim for maximum privacy and security, this project is still in active development. Please do not rely on it for highly sensitive communications until thoroughly audited.

---

**Built with ❤️ for the decentralized web.**
