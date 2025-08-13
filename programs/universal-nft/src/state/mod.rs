use anchor_lang::prelude::*;

/// Global configuration for the universal NFT program
#[account]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub zetachain_gateway: Pubkey,
    pub collection_authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub cross_chain_fee: u64,
    pub bump: u8,
}

impl Space for GlobalConfig {
    const INIT_SPACE: usize = 8 + // discriminator
        32 + // authority
        32 + // zetachain_gateway
        32 + // collection_authority
        32 + // fee_recipient
        8 + // cross_chain_fee
        1; // bump
}

/// Represents a cross-chain NFT with ZetaChain integration
#[account]
pub struct UniversalNft {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub original_chain: String,
    pub original_contract: String,
    pub original_token_id: String,
    pub metadata_uri: String,
    pub is_locked: bool,
    pub lock_destination_chain: String,
    pub lock_recipient: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl Space for UniversalNft {
    const INIT_SPACE: usize = 8 + // discriminator
        32 + // mint
        32 + // owner
        4 + 32 + // original_chain (max 32 chars)
        4 + 64 + // original_contract (max 64 chars)
        4 + 32 + // original_token_id (max 32 chars)
        4 + 200 + // metadata_uri (max 200 chars)
        1 + // is_locked
        4 + 32 + // lock_destination_chain (max 32 chars)
        4 + 64 + // lock_recipient (max 64 chars)
        8 + // created_at
        8 + // updated_at
        1; // bump
}

/// Cross-chain transfer request pending confirmation
#[account]
pub struct CrossChainTransfer {
    pub transfer_id: String,
    pub nft_mint: Pubkey,
    pub source_owner: Pubkey,
    pub destination_chain: String,
    pub destination_recipient: String,
    pub status: TransferStatus,
    pub initiated_at: i64,
    pub completed_at: Option<i64>,
    pub bump: u8,
}

impl Space for CrossChainTransfer {
    const INIT_SPACE: usize = 8 + // discriminator
        4 + 32 + // transfer_id (max 32 chars)
        32 + // nft_mint
        32 + // source_owner
        4 + 32 + // destination_chain (max 32 chars)
        4 + 64 + // destination_recipient (max 64 chars)
        1 + // status
        8 + // initiated_at
        1 + 8 + // completed_at (Option<i64>)
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TransferStatus {
    Initiated,
    Confirmed,
    Completed,
    Failed,
}

/// Seeds for PDAs
pub const GLOBAL_CONFIG_SEED: &[u8] = b"global_config";
pub const UNIVERSAL_NFT_SEED: &[u8] = b"universal_nft";
pub const CROSS_CHAIN_TRANSFER_SEED: &[u8] = b"cross_chain_transfer";
