pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("73ce2AD3AZpaGFNcdavnbKbhNGSmz3PNyv2GCDM3Yy3c");

#[program]
pub mod universal_nft {
    use super::*;

    /// Initialize the global configuration for the universal NFT program
    pub fn initialize(
        ctx: Context<Initialize>,
        bump: u8,
        cross_chain_fee: Option<u64>,
    ) -> Result<()> {
        initialize::handler(ctx, bump, cross_chain_fee)
    }

    /// Mint a new universal NFT with cross-chain metadata
    pub fn mint_nft(
        ctx: Context<MintNft>,
        bump: u8,
        name: String,
        symbol: String,
        uri: String,
        original_chain: String,
        original_contract: String,
        original_token_id: String,
    ) -> Result<()> {
        mint_nft::handler(ctx, bump, name, symbol, uri, original_chain, original_contract, original_token_id)
    }

    /// Initiate a cross-chain transfer of an NFT
    pub fn initiate_cross_chain_transfer(
        ctx: Context<InitiateCrossChainTransfer>,
        transfer_id: String,
        destination_chain: String,
        destination_recipient: String,
        bump: u8,
    ) -> Result<()> {
        initiate_cross_chain_transfer::handler(ctx, transfer_id, destination_chain, destination_recipient, bump)
    }

    /// Confirm a cross-chain transfer (called by ZetaChain gateway)
    pub fn confirm_cross_chain_transfer(
        ctx: Context<ConfirmCrossChainTransfer>,
        transfer_id: String,
    ) -> Result<()> {
        confirm_cross_chain_transfer::handler(ctx, transfer_id)
    }

    /// Complete a cross-chain transfer by burning the NFT
    pub fn complete_cross_chain_transfer(
        ctx: Context<CompleteCrossChainTransfer>,
        transfer_id: String,
    ) -> Result<()> {
        complete_cross_chain_transfer::handler(ctx, transfer_id)
    }

    /// Called by ZetaChain gateway when receiving cross-chain NFT transfer
    pub fn on_call(
        ctx: Context<OnCall>,
        amount: u64,
        sender: [u8; 20],
        data: Vec<u8>,
        bump: u8,
    ) -> Result<()> {
        on_call::handler(ctx, amount, sender, data, bump)
    }

    /// Called by ZetaChain gateway when cross-chain transfer fails
    pub fn on_revert(
        ctx: Context<OnRevert>,
        amount: u64,
        sender: Pubkey,
        data: Vec<u8>,
        transfer_id: String,
    ) -> Result<()> {
        on_revert::handler(ctx, amount, sender, data, transfer_id)
    }

    /// Initiate cross-chain transfer via ZetaChain gateway
    pub fn deposit_and_call(
        ctx: Context<DepositAndCall>,
        transfer_id: String,
        destination_chain_id: u64,
        destination_recipient: [u8; 20],
        revert_options: Option<gateway::RevertOptions>,
        bump: u8,
    ) -> Result<()> {
        deposit_and_call::handler(ctx, transfer_id, destination_chain_id, destination_recipient, revert_options, bump)
    }
}
