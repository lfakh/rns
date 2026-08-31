# RNS

A Rust project with Svelte frontend leveraging Reticulum Network Service and Yggdrasil for peer-to-peer privacy and distributed networking.

## Overview

RNS is a decentralized application that combines Rust backend with Svelte frontend components, built on top of Reticulum Network Service (RNS) and Yggdrasil mesh networking for maximum peer-to-peer privacy and open web3 integration.

## Tech Stack

- **Rust** - Backend and core logic
- **Svelte** - Frontend framework
- **Astro** - Static site generation
- **Reticulum Network Service (RNS)** - Application-agnostic routing protocol
- **Yggdrasil** - IPv6 mesh network for end-to-end encrypted communications
- **JavaScript** - Supporting scripts
- **CSS** - Styling

## Getting Started

### Prerequisites

- Rust
- Node.js/npm or yarn
- Yggdrasil (for mesh networking)

### Installation

```bash
# Clone the repository
git clone https://github.com/lfakh/rns.git
cd rns

# Install dependencies
npm install  # or yarn install

# Build the project
cargo build
```

### Development

```bash
# Run development server
npm run dev

# Build for production
cargo build --release
npm run build
```

## Contributing

Contributions are welcome! We're looking to implement several key features to enhance RNS. Here are some areas we're actively seeking help with:

### High-Priority Features for Contributors

#### 1. **OrbitDB Integration for Distributed Database**
   - Implement OrbitDB as a peer-to-peer distributed database layer
   - Enable decentralized data storage without relying on centralized servers
   - Create Rust bindings or WASM integration with OrbitDB
   - **Impact**: Provides a fully decentralized data persistence layer that complements RNS routing

#### 2. **Web3 User Discovery System**
   - Build a decentralized user discovery mechanism leveraging web3 protocols
   - Integrate with IPFS/DHT for peer discovery
   - Create identity management tied to cryptographic keys
   - Implement privacy-preserving user lookup without exposing network topology
   - **Impact**: Enables users to find and connect with each other in a fully decentralized manner

#### 3. **Yggdrasil Network Integration (Recommended Starting Point)**
   - Integrate Yggdrasil mesh networking stack with RNS
   - Implement automatic peer discovery via Yggdrasil coordinates
   - Create network status monitoring and connection management UI
   - **Why start here?**: Yggdrasil and RNS work together naturally for end-to-end encrypted routing

### Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make your changes and test thoroughly
4. Submit a Pull Request with a clear description of your changes

### Development Guidelines

- Follow Rust naming conventions and best practices
- Write tests for new functionality
- Update documentation as needed
- Ensure code builds without warnings

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
