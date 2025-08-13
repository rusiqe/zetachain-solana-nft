use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(transfer_id: String, destination_chain: String, destination_recipient: String)]
pub struct InitiateCrossChainTransfer<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
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
        constraint = universal_nft.owner == owner.key() @ ErrorCode::Unauthorized,
        constraint = !universal_nft.is_locked @ ErrorCode::NftLocked,
    )]
    pub universal_nft: Account<'info, UniversalNft>,
    
    #[account(
        init,
        payer = payer,
        space = CrossChainTransfer::INIT_SPACE,
        seeds = [
            CROSS_CHAIN_TRANSFER_SEED,
            transfer_id.as_bytes(),
        ],
        bump
    )]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = owner_token_account.mint == nft_mint.key() @ ErrorCode::InvalidOriginalChain,
        constraint = owner_token_account.owner == owner.key() @ ErrorCode::Unauthorized,
        constraint = owner_token_account.amount == 1 @ ErrorCode::Unauthorized,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: This will be validated by ZetaChain gateway
    pub zetachain_gateway: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitiateCrossChainTransfer>,
    transfer_id: String,
    destination_chain: String,
    destination_recipient: String,
    bump: u8,
) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let universal_nft = &mut ctx.accounts.universal_nft;
    let cross_chain_transfer = &mut ctx.accounts.cross_chain_transfer;
    let clock = Clock::get()?;
    
    // Validate input parameters
    require!(!transfer_id.is_empty() && transfer_id.len() <= 32, ErrorCode::InvalidDestinationAddress);
    require!(!destination_chain.is_empty() && destination_chain.len() <= 32, ErrorCode::InvalidChainId);
    require!(!destination_recipient.is_empty() && destination_recipient.len() <= 64, ErrorCode::InvalidDestinationAddress);
    
    // Validate ZetaChain gateway
    require!(
        ctx.accounts.zetachain_gateway.key() == global_config.zetachain_gateway,
        ErrorCode::GatewayNotConfigured
    );
    
    // Check if payer has enough funds for cross-chain fee
    require!(
        ctx.accounts.payer.lamports() >= global_config.cross_chain_fee,
        ErrorCode::InsufficientFunds
    );
    
    // Lock the NFT for cross-chain transfer
    universal_nft.is_locked = true;
    universal_nft.lock_destination_chain = destination_chain.clone();
    universal_nft.lock_recipient = destination_recipient.clone();
    universal_nft.updated_at = clock.unix_timestamp;
    
    // Initialize cross-chain transfer state
    cross_chain_transfer.transfer_id = transfer_id.clone();
    cross_chain_transfer.nft_mint = ctx.accounts.nft_mint.key();
    cross_chain_transfer.source_owner = ctx.accounts.owner.key();
    cross_chain_transfer.destination_chain = destination_chain.clone();
    cross_chain_transfer.destination_recipient = destination_recipient.clone();
    cross_chain_transfer.status = TransferStatus::Initiated;
    cross_chain_transfer.initiated_at = clock.unix_timestamp;
    cross_chain_transfer.completed_at = None;
    cross_chain_transfer.bump = bump;
    
    // Transfer fee to fee recipient
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.payer.key(),
        &global_config.fee_recipient,
        global_config.cross_chain_fee,
    );
    
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.zetachain_gateway.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    
    // Emit cross-chain message to ZetaChain
    // Note: In a real implementation, this would interact with ZetaChain's messaging protocol
    msg!(
        "CrossChainTransferInitiated: transfer_id={}, mint={}, destination_chain={}, recipient={}, original_chain={}, original_token_id={}",
        transfer_id,
        ctx.accounts.nft_mint.key(),
        destination_chain,
        destination_recipient,
        universal_nft.original_chain,
        universal_nft.original_token_id
    );
    
    Ok(())
}
