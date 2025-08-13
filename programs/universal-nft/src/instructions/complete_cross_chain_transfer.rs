use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token, TokenAccount, Mint};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(transfer_id: String)]
pub struct CompleteCrossChainTransfer<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
        constraint = global_config.zetachain_gateway == zetachain_gateway.key() @ ErrorCode::GatewayNotConfigured
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(
        mut,
        seeds = [
            UNIVERSAL_NFT_SEED,
            nft_mint.key().as_ref(),
        ],
        bump = universal_nft.bump,
        constraint = universal_nft.mint == nft_mint.key() @ ErrorCode::InvalidOriginalChain,
        constraint = universal_nft.is_locked @ ErrorCode::InvalidTransferStatus,
    )]
    pub universal_nft: Account<'info, UniversalNft>,
    
    #[account(
        mut,
        seeds = [
            CROSS_CHAIN_TRANSFER_SEED,
            transfer_id.as_bytes(),
        ],
        bump = cross_chain_transfer.bump,
        constraint = cross_chain_transfer.nft_mint == nft_mint.key() @ ErrorCode::InvalidOriginalChain,
        constraint = cross_chain_transfer.status == TransferStatus::Confirmed @ ErrorCode::InvalidTransferStatus,
    )]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = owner_token_account.mint == nft_mint.key() @ ErrorCode::InvalidOriginalChain,
        constraint = owner_token_account.owner == cross_chain_transfer.source_owner @ ErrorCode::Unauthorized,
        constraint = owner_token_account.amount == 1 @ ErrorCode::Unauthorized,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Must be the collection authority to burn NFTs
    #[account(
        constraint = collection_authority.key() == global_config.collection_authority @ ErrorCode::Unauthorized
    )]
    pub collection_authority: Signer<'info>,
    
    /// CHECK: This is the ZetaChain gateway - validated by global config
    pub zetachain_gateway: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<CompleteCrossChainTransfer>,
    transfer_id: String,
) -> Result<()> {
    let universal_nft = &mut ctx.accounts.universal_nft;
    let cross_chain_transfer = &mut ctx.accounts.cross_chain_transfer;
    let clock = Clock::get()?;
    
    // Burn the NFT token to complete the cross-chain transfer
    let cpi_accounts = Burn {
        mint: ctx.accounts.nft_mint.to_account_info(),
        from: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.collection_authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    burn(cpi_ctx, 1)?;
    
    // Update transfer status
    cross_chain_transfer.status = TransferStatus::Completed;
    cross_chain_transfer.completed_at = Some(clock.unix_timestamp);
    
    // Update NFT state
    universal_nft.is_locked = false;
    universal_nft.updated_at = clock.unix_timestamp;
    
    msg!(
        "CrossChainTransferCompleted: transfer_id={}, mint={}, destination_chain={}, recipient={}",
        transfer_id,
        ctx.accounts.nft_mint.key(),
        cross_chain_transfer.destination_chain,
        cross_chain_transfer.destination_recipient
    );
    
    Ok(())
}
