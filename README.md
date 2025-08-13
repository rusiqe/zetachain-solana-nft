# ZetaChain Universal NFT Program

A Solana program that enables universal NFT functionality with robust cross-chain transfers and interactions through ZetaChain's interoperability protocol.

## 🌟 Features

- **Universal NFT Minting**: Create NFTs with cross-chain metadata tracking original chain, contract, and token ID
- **Cross-Chain Transfers**: Initiate, confirm, and complete NFT transfers across different blockchains
- **ZetaChain Integration**: Built-in gateway integration for seamless cross-chain messaging
- **Advanced Security**: Comprehensive ownership verification, transfer locks, and fee management
- **Solana Optimized**: Efficient compute budget usage, rent exemption handling, and account structure
- **Multi-Chain Support**: Compatible with Ethereum, Polygon, BSC, and other EVM chains

## 🏗️ Architecture

### Core Components

1. **GlobalConfig**: Program-wide configuration including authorities, gateway addresses, and fees
2. **UniversalNft**: Individual NFT accounts with cross-chain metadata and locking mechanisms
3. **CrossChainTransfer**: Transfer state management for cross-chain operations

### Key Instructions

**Core NFT Operations:**
- `initialize()`: Set up global program configuration
- `mint_nft()`: Create universal NFTs with cross-chain metadata

**Manual Transfer Pattern:**
- `initiate_cross_chain_transfer()`: Start cross-chain transfer process
- `confirm_cross_chain_transfer()`: Gateway confirmation of transfer
- `complete_cross_chain_transfer()`: Finalize transfer by burning source NFT

**ZetaChain Gateway Integration Pattern:**
- `on_call()`: Receive cross-chain NFT transfers via gateway callbacks
- `on_revert()`: Handle failed cross-chain transfers with automatic unlock
- `deposit_and_call()`: Initiate cross-chain transfers via direct gateway CPI

## 🔧 Technical Specifications

### Account Structure

```rust
pub struct GlobalConfig {
    pub authority: Pubkey,              // Program authority
    pub zetachain_gateway: Pubkey,      // ZetaChain gateway address
    pub collection_authority: Pubkey,   // NFT collection authority
    pub fee_recipient: Pubkey,          // Cross-chain fee recipient
    pub cross_chain_fee: u64,          // Fee in lamports
    pub bump: u8,                      // PDA bump
}

pub struct UniversalNft {
    pub mint: Pubkey,                   // NFT mint address
    pub owner: Pubkey,                  // Current owner
    pub original_chain: String,         // Original blockchain
    pub original_contract: String,      // Original contract address
    pub original_token_id: String,      // Original token ID
    pub metadata_uri: String,           // Metadata URI
    pub is_locked: bool,                // Transfer lock status
    pub lock_destination_chain: String, // Locked for transfer to chain
    pub lock_recipient: String,         // Locked for transfer to recipient
    pub created_at: i64,               // Creation timestamp
    pub updated_at: i64,               // Last update timestamp
    pub bump: u8,                      // PDA bump
}

pub struct CrossChainTransfer {
    pub transfer_id: String,            // Unique transfer identifier
    pub nft_mint: Pubkey,              // NFT being transferred
    pub source_owner: Pubkey,          // Original owner
    pub destination_chain: String,      // Target blockchain
    pub destination_recipient: String,  // Target recipient address
    pub status: TransferStatus,         // Transfer status
    pub initiated_at: i64,             // Initiation timestamp
    pub completed_at: Option<i64>,     // Completion timestamp
    pub bump: u8,                      // PDA bump
}
```

### Error Handling

Comprehensive error codes for various failure scenarios:
- Insufficient funds for operations
- NFT locked during transfer
- Invalid chain IDs or addresses
- Unauthorized operations
- Gateway configuration errors
- Transfer timeout scenarios

## 🚀 Getting Started

### Prerequisites

- Rust 1.75.0+
- Solana CLI 1.18.0+
- Anchor Framework 0.31.1+
- Node.js 18+
- Yarn/NPM

### Installation

```bash
# Clone the repository
git clone https://github.com/rusiqe/zetachain-solana-nft
cd zetachain-solana-nft/universal-nft

# Install dependencies
yarn install

# Build the program
anchor build

# Run tests
anchor test
```

### Local Development

1. Start local Solana test validator:
```bash
solana-test-validator
```

2. Deploy program:
```bash
anchor deploy
```

3. Run integration tests:
```bash
anchor test --skip-local-validator
```

## 📋 Usage Examples

### Initialize Global Configuration

```typescript
const [globalConfigPda, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from("global_config")],
  program.programId
);

await program.methods
  .initialize(
    bump,
    new BN(10_000) // Cross-chain fee in lamports
  )
  .accounts({
    globalConfig: globalConfigPda,
    authority: authorityKeypair.publicKey,
    zetachainGateway: gatewayKeypair.publicKey,
    collectionAuthority: collectionKeypair.publicKey,
    feeRecipient: feeRecipientKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([authorityKeypair])
  .rpc();
```

### Mint Universal NFT

```typescript
const nftMint = Keypair.generate();
const [universalNftPda, nftBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("universal_nft"), nftMint.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .mintNft(
    nftBump,
    "ZetaChain Universal NFT",
    "ZUNFT",
    "https://zetachain.com/metadata/1.json",
    "ethereum",
    "0x1234567890abcdef1234567890abcdef12345678",
    "1"
  )
  .accounts({
    globalConfig: globalConfigPda,
    universalNft: universalNftPda,
    mint: nftMint.publicKey,
    tokenAccount: tokenAccount,
    payer: payerKeypair.publicKey,
    recipient: recipientKeypair.publicKey,
    collectionAuthority: collectionKeypair.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .signers([payerKeypair, nftMint, collectionKeypair])
  .rpc();
```

### Initiate Cross-Chain Transfer

```typescript
const transferId = "unique_transfer_123";
const [crossChainTransferPda, transferBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("cross_chain_transfer"), Buffer.from(transferId)],
  program.programId
);

await program.methods
  .initiateCrossChainTransfer(
    transferId,
    "polygon",
    "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    transferBump
  )
  .accounts({
    globalConfig: globalConfigPda,
    universalNft: universalNftPda,
    crossChainTransfer: crossChainTransferPda,
    nftMint: nftMint.publicKey,
    ownerTokenAccount: ownerTokenAccount,
    owner: ownerKeypair.publicKey,
    payer: payerKeypair.publicKey,
    zetachainGateway: gatewayKeypair.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .signers([payerKeypair, ownerKeypair])
  .rpc();
```

## 🔐 Security Features

- **PDA-based Account Security**: All program accounts use Program Derived Addresses
- **Authority Validation**: Multi-level authority checks for different operations
- **Transfer Locking**: NFTs are locked during cross-chain transfers
- **Fee Management**: Configurable fees for cross-chain operations
- **Gateway Authentication**: Only authorized ZetaChain gateway can confirm transfers
- **Ownership Verification**: Strict ownership checks before allowing transfers

## 🌉 ZetaChain Gateway Integration

### Integration Patterns

This program supports **two integration patterns** for maximum flexibility:

#### 1. **Manual Transfer Pattern** (Direct Control)
Best for applications that need fine-grained control over the transfer process:

```typescript
// 1. Initiate transfer
await program.methods.initiateCrossChainTransfer(...)
// 2. Gateway confirms (external)
// 3. Complete transfer  
await program.methods.completeCrossChainTransfer(...)
```

#### 2. **Gateway Callback Pattern** (Full ZetaChain Integration)
Best for seamless integration with ZetaChain's protocol contracts:

```typescript
// Outgoing: Send NFT to another chain
await program.methods.depositAndCall(
  ctx,
  transferId,
  destinationChainId, // e.g., 1 for Ethereum
  destinationRecipient, // [u8; 20] Ethereum address
  revertOptions,
  bump
);

// Incoming: Receive NFT from another chain (called by gateway)
await program.methods.onCall(
  ctx,
  amount,
  sender, // [u8; 20] source chain address
  data, // NFT metadata: "chain:ethereum,token_id:123,uri:https://..."
  bump
);

// Revert: Handle failed transfers (called by gateway)
await program.methods.onRevert(
  ctx,
  amount,
  sender,
  data, // Error reason
  transferId
);
```

### Gateway Security

**Caller Validation:**
```rust
// Validate that only the configured gateway can call these functions
let current_ix = get_instruction_relative(0, &instruction_sysvar_account)?;
require!(
    current_ix.program_id == global_config.zetachain_gateway,
    ErrorCode::Unauthorized
);
```

**Cross-Chain Message Format:**
```rust
// Structured NFT data format for cross-chain transfers
"chain:ethereum,token_id:123,uri:https://metadata.com/1.json,name:MyNFT,symbol:MNFT"
```

### Supported Chains

- **Ethereum** (Chain ID: 1)
- **Polygon** (Chain ID: 137) 
- **BSC** (Chain ID: 56)
- **ZetaChain** (Chain ID: 7000)
- **Solana** (Chain ID: 900)

### Cross-Chain Workflow

#### **Sending NFT from Solana:**
1. User calls `deposit_and_call()`
2. Program burns local NFT
3. Program calls gateway with NFT metadata
4. Gateway routes to destination chain
5. Destination chain mints equivalent NFT

#### **Receiving NFT to Solana:**
1. Source chain burns/locks NFT
2. ZetaChain gateway calls our `on_call()`
3. Program validates gateway caller
4. Program parses NFT metadata
5. Program mints new NFT to recipient

#### **Failed Transfer Handling:**
1. Destination chain rejects transfer
2. ZetaChain gateway calls our `on_revert()`
3. Program unlocks NFT (if locked)
4. Program updates transfer status to Failed

## 🧪 Testing

Comprehensive test suite covering:

- ✅ Global configuration initialization
- ✅ Universal NFT minting
- ✅ Cross-chain transfer initiation
- ✅ Transfer confirmation and completion
- ✅ Error scenarios and edge cases
- ✅ Fee collection and authority validation

```bash
# Run all tests
anchor test

# Run specific test
anchor test --grep "Mints a universal NFT"
```

## 📊 Performance Optimizations

- **Compute Budget**: Optimized for maximum 400,000 compute units
- **Rent Exemption**: Efficient account sizing for rent exemption
- **Account Packing**: Minimal account space usage
- **PDA Optimization**: Efficient seed generation for PDAs

## 🛠️ Development Roadmap

### Phase 1: Core Functionality ✅
- [x] Basic NFT minting and transfer
- [x] Cross-chain metadata tracking
- [x] ZetaChain gateway integration

### Phase 2: Advanced Features
- [ ] Metaplex metadata integration
- [ ] Batch operations for multiple NFTs
- [ ] Advanced fee structures

### Phase 3: Production Ready
- [ ] Mainnet deployment scripts
- [ ] Comprehensive documentation
- [ ] SDK and client libraries

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Support

- **Documentation**: Check the inline code documentation
- **Issues**: Report bugs via GitHub Issues
- **Community**: Join ZetaChain Discord for community support

## 🙏 Acknowledgments

- **ZetaChain Team**: For the universal blockchain interoperability vision
- **Solana Foundation**: For the high-performance blockchain platform
- **Anchor Framework**: For the excellent Solana development framework

---

Built with ❤️ for universal blockchain interoperability
