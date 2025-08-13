use anchor_lang::prelude::*;
use anchor_lang::solana_program::{sysvar, sysvar::instructions::get_instruction_relative};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(
        mut,
        seeds = [
            UNIVERSAL_NFT_SEED,
            mint.key().as_ref(),
        ],
        bump = universal_nft.bump,
    )]
    pub universal_nft: Account<'info, UniversalNft>,
    
    #[account(
        mut,
        seeds = [
            CROSS_CHAIN_TRANSFER_SEED,
            transfer_id.as_bytes(),
        ],
        bump = cross_chain_transfer.bump,
        constraint = cross_chain_transfer.nft_mint == mint.key() @ ErrorCode::InvalidOriginalChain,
    )]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: This is the ZetaChain gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    
    /// CHECK: Instruction sysvar for gateway validation
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
}

/// Called by ZetaChain gateway when a cross-chain NFT transfer fails
pub fn handler(
    ctx: Context<OnRevert>,
    amount: u64,
    sender: Pubkey,
    data: Vec<u8>,
    transfer_id: String,
) -> Result<()> {
    // Validate that caller is the ZetaChain gateway
    let current_ix = get_instruction_relative(
        0,
        &ctx.accounts.instruction_sysvar_account.to_account_info(),
    )
    .unwrap();
    
    require!(
        current_ix.program_id == ctx.accounts.global_config.zetachain_gateway,
        ErrorCode::Unauthorized
    );
    
    let universal_nft = &mut ctx.accounts.universal_nft;
    let cross_chain_transfer = &mut ctx.accounts.cross_chain_transfer;
    let clock = Clock::get()?;
    
    // Parse revert reason from data
    let revert_reason = String::from_utf8(data)
        .unwrap_or_else(|_| "Unknown revert reason".to_string());
    
    // Unlock the NFT since the transfer failed
    universal_nft.is_locked = false;
    universal_nft.lock_destination_chain = String::new();
    universal_nft.lock_recipient = String::new();
    universal_nft.updated_at = clock.unix_timestamp;
    
    // Update transfer status to failed
    cross_chain_transfer.status = TransferStatus::Failed;
    cross_chain_transfer.completed_at = Some(clock.unix_timestamp);
    
    msg!(
        "Cross-chain NFT transfer reverted: transfer_id={}, mint={}, reason={}",
        transfer_id,
        ctx.accounts.mint.key(),
        revert_reason
    );
    
    Ok(())
}
