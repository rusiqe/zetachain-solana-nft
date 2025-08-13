# ZetaChain Gateway Integration Guide

This document provides detailed information about integrating with ZetaChain's protocol-contracts-solana for universal NFT transfers.

## 🏗️ Integration Architecture

### Two Integration Patterns

Our Universal NFT program supports **two distinct patterns** to accommodate different use cases:

#### Pattern 1: Manual Transfer Control
- Direct program-to-program interactions
- Fine-grained control over each step
- Custom validation and business logic
- Best for complex workflows

#### Pattern 2: ZetaChain Gateway Integration
- Automatic callback-driven transfers
- Seamless integration with ZetaChain protocol
- Standardized cross-chain messaging
- Best for simple, reliable transfers

## 🔄 Gateway Callback Functions

### `on_call()` - Receiving Cross-Chain NFTs

Called automatically by the ZetaChain gateway when an NFT is sent to Solana from another chain.

```rust
pub fn on_call(
    ctx: Context<OnCall>,
    amount: u64,              // SOL amount transferred with call
    sender: [u8; 20],         // Ethereum-style address of sender
    data: Vec<u8>,           // Encoded NFT metadata
    bump: u8,                // PDA bump seed
) -> Result<()>
```

**Function Flow:**
1. **Gateway Validation** - Ensures caller is authorized ZetaChain gateway
2. **Data Parsing** - Extracts NFT metadata from cross-chain message
3. **NFT Minting** - Creates new NFT with original chain provenance
4. **State Tracking** - Records cross-chain transfer details

**Data Format Expected:**
```
"chain:ethereum,token_id:123,uri:https://metadata.com/1.json,name:MyNFT,symbol:MNFT"
```

### `on_revert()` - Handling Failed Transfers

Called by the gateway when a cross-chain transfer fails on the destination chain.

```rust
pub fn on_revert(
    ctx: Context<OnRevert>,
    amount: u64,              // SOL amount being reverted
    sender: Pubkey,           // Original sender address
    data: Vec<u8>,           // Revert reason/error message
    transfer_id: String,      // Transfer identifier
) -> Result<()>
```

**Function Flow:**
1. **Gateway Validation** - Confirms call from authorized gateway
2. **NFT Unlocking** - Releases any locked NFTs
3. **Status Update** - Marks transfer as failed
4. **State Cleanup** - Resets transfer-related state

### `deposit_and_call()` - Initiating Outbound Transfers

User-callable function to send NFTs from Solana to other chains via the gateway.

```rust
pub fn deposit_and_call(
    ctx: Context<DepositAndCall>,
    transfer_id: String,              // Unique transfer identifier
    destination_chain_id: u64,        // Target chain ID (1=Ethereum, 137=Polygon)
    destination_recipient: [u8; 20],  // Ethereum-style recipient address
    revert_options: Option<gateway::RevertOptions>, // Failure handling options
    bump: u8,                         // PDA bump seed
) -> Result<()>
```

**Function Flow:**
1. **Validation** - Checks NFT ownership, chain validity, etc.
2. **NFT Burning** - Burns the NFT since it's moving to another chain
3. **Gateway CPI** - Calls ZetaChain gateway with transfer details
4. **State Tracking** - Records outbound transfer attempt

## 🛡️ Security Implementation

### Gateway Caller Validation

Critical security feature that ensures only the authorized ZetaChain gateway can call callback functions:

```rust
use anchor_lang::solana_program::{sysvar, sysvar::instructions::get_instruction_relative};

// Get the current instruction to check who called us
let current_ix = get_instruction_relative(
    0,
    &ctx.accounts.instruction_sysvar_account.to_account_info(),
)?;

// Ensure the caller is the configured gateway
require!(
    current_ix.program_id == ctx.accounts.global_config.zetachain_gateway,
    ErrorCode::Unauthorized
);
```

### Multi-Level Authority Checks

```rust
// Program authority for administrative functions
global_config.authority

// Collection authority for NFT minting/burning  
global_config.collection_authority

// Gateway authority for cross-chain operations
global_config.zetachain_gateway
```

### Transfer State Management

```rust
pub enum TransferStatus {
    Initiated,  // Transfer request created
    Confirmed,  // Gateway acknowledged
    Completed,  // Successfully transferred
    Failed,     // Transfer reverted/failed
}
```

## 📡 Cross-Chain Message Protocol

### Outbound Message Format

When sending NFTs from Solana to other chains:

```rust
let message_data = format!(
    "chain:{},token_id:{},uri:{},name:{},symbol:{}",
    universal_nft.original_chain,    // "ethereum" 
    universal_nft.original_token_id, // "123"
    universal_nft.metadata_uri,      // "https://..."
    "UniversalNFT",                  // NFT name
    "UNFT"                           // NFT symbol
);
```

### Inbound Message Parsing

When receiving NFTs on Solana from other chains:

```rust
fn parse_cross_chain_nft_data(data: &[u8]) -> Result<CrossChainNftData> {
    let message = String::from_utf8(data.to_vec())?;
    let parts: Vec<&str> = message.split(',').collect();
    
    // Parse key-value pairs
    for part in parts {
        let kv: Vec<&str> = part.split(':').collect();
        match kv[0] {
            "chain" => nft_data.original_chain = kv[1].to_string(),
            "token_id" => nft_data.token_id = kv[1].to_string(),
            "uri" => nft_data.metadata_uri = kv[1].to_string(),
            // ... parse other fields
        }
    }
}
```

## 🔗 Gateway Integration Examples

### Example 1: Sending NFT from Solana to Ethereum

```typescript
// User owns an NFT and wants to send it to Ethereum
const transferId = `transfer_${Date.now()}`;
const ethereumRecipient = [/* 20-byte Ethereum address */];

await program.methods
  .depositAndCall(
    transferId,
    1, // Ethereum chain ID
    ethereumRecipient,
    {
      revertAddress: userWallet.publicKey.toBytes(),
      callOnRevert: true,
      revertMessage: "Transfer failed",
    },
    bump
  )
  .accounts({
    globalConfig: globalConfigPda,
    universalNft: universalNftPda,
    crossChainTransfer: transferPda,
    nftMint: nftMint.publicKey,
    ownerTokenAccount: tokenAccount,
    owner: userWallet.publicKey,
    payer: userWallet.publicKey,
    gatewayPda: zetaGatewayPda,
    gatewayProgram: zetaGatewayProgram,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .signers([userWallet])
  .rpc();
```

### Example 2: Gateway Calling on_call (Receiving NFT)

```typescript
// This would be called by the ZetaChain gateway program
const nftMetadata = Buffer.from(
  "chain:ethereum,token_id:123,uri:https://metadata.example.com/123,name:CoolNFT,symbol:COOL"
);

await gatewayProgram.methods
  .executeCall(
    universalNftProgram.programId,
    amount,
    senderEthAddress, // [u8; 20]
    nftMetadata,
    // ... other gateway parameters
  )
  .accounts({
    // Gateway accounts...
    targetProgram: universalNftProgram.programId,
  })
  .rpc();
```

## ⚙️ Configuration Requirements

### Global Config Setup

```typescript
const globalConfigData = {
  authority: programAuthority.publicKey,
  zetachainGateway: zetaGatewayProgram.programId, // Critical: Must be actual gateway
  collectionAuthority: nftMintAuthority.publicKey,
  feeRecipient: feeCollector.publicKey,
  crossChainFee: new BN(10_000), // 0.00001 SOL
};
```

### Required Accounts

Every gateway interaction requires:

```rust
#[account(address = sysvar::instructions::id())]
pub instruction_sysvar_account: UncheckedAccount<'info>,
```

This enables the security validation that ensures only the gateway can call protected functions.

## 🧪 Testing Gateway Integration

### Mock Gateway for Testing

```typescript
// Create a test program that mimics gateway behavior
const mockGateway = await createMockGateway();

// Test on_call functionality
await mockGateway.methods
  .callConnectedProgram(
    universalNftProgram.programId,
    "on_call",
    amount,
    senderAddress,
    nftMetadata
  )
  .rpc();
```

### Integration Test Scenarios

1. **Successful Cross-Chain Mint**
   - Gateway calls `on_call` with valid NFT data
   - Verify NFT is minted with correct metadata
   - Check provenance tracking is accurate

2. **Failed Transfer Revert**
   - Gateway calls `on_revert` for failed transfer
   - Verify NFT is unlocked if previously locked
   - Check transfer status updated to Failed

3. **Unauthorized Caller Rejection**
   - Non-gateway program attempts to call `on_call`
   - Verify transaction fails with Unauthorized error
   - Ensure no state changes occur

## 🔄 Migration from Manual to Gateway Pattern

### Gradual Migration Strategy

1. **Deploy Enhanced Program** - Add gateway functions alongside existing ones
2. **Test Integration** - Verify gateway callbacks work correctly  
3. **Update Client Code** - Migrate to `deposit_and_call` for new transfers
4. **Monitor Both Patterns** - Maintain compatibility during transition
5. **Deprecate Manual Pattern** - Eventually phase out manual transfers

### Backward Compatibility

The enhanced program maintains full backward compatibility:

```rust
// Old manual pattern still works
pub fn initiate_cross_chain_transfer(...) -> Result<()> { ... }
pub fn confirm_cross_chain_transfer(...) -> Result<()> { ... } 
pub fn complete_cross_chain_transfer(...) -> Result<()> { ... }

// New gateway pattern available
pub fn on_call(...) -> Result<()> { ... }
pub fn on_revert(...) -> Result<()> { ... }
pub fn deposit_and_call(...) -> Result<()> { ... }
```

## 🚨 Common Integration Issues

### Issue 1: Gateway Address Mismatch
**Problem:** Using wrong gateway program ID in configuration
**Solution:** Verify gateway address matches ZetaChain's deployed gateway

### Issue 2: Missing Instruction Sysvar
**Problem:** Forgot to include instruction sysvar account
**Solution:** Always include `instruction_sysvar_account` in gateway callbacks

### Issue 3: Invalid Message Format
**Problem:** Cross-chain message doesn't match expected format
**Solution:** Use standardized format: `"chain:X,token_id:Y,uri:Z,name:A,symbol:B"`

### Issue 4: Insufficient Compute Budget
**Problem:** Complex cross-chain operations exceed compute limits
**Solution:** Use compute budget instruction or optimize account access patterns

## 📈 Performance Considerations

### Compute Unit Usage

| Operation | Approximate CU Cost |
|-----------|-------------------|
| `on_call` with mint | ~200,000 CU |
| `deposit_and_call` | ~150,000 CU |
| `on_revert` | ~50,000 CU |

### Account Size Optimization

```rust
// Optimized account sizes for rent exemption
impl Space for GlobalConfig {
    const INIT_SPACE: usize = 145; // Minimal size
}

impl Space for UniversalNft {
    const INIT_SPACE: usize = 456; // Includes all metadata
}
```

## 🔮 Future Enhancements

### Planned Features

1. **Batch Operations** - Transfer multiple NFTs in single transaction
2. **Collection Support** - Verify NFTs belong to specific collections
3. **Metadata Validation** - Ensure metadata URIs are accessible
4. **Fee Optimization** - Dynamic fee calculation based on destination chain
5. **Event Streaming** - Real-time cross-chain transfer notifications

### Integration Roadmap

- **Phase 1**: Basic gateway integration (✅ Complete)
- **Phase 2**: Advanced metadata parsing and validation
- **Phase 3**: Collection and royalty support
- **Phase 4**: Optimized batch operations
- **Phase 5**: Production monitoring and analytics

---

This integration guide provides the foundation for building robust cross-chain NFT applications using ZetaChain's interoperability protocol with our Universal NFT program.
