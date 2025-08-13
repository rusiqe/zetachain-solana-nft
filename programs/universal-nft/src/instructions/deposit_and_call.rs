use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token, TokenAccount, Mint};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
pub struct DepositAndCall<'info> {
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
    
    #[account(mut)]
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
    
    /// CHECK: This is the ZetaChain gateway PDA
    #[account(mut)]
    pub gateway_pda: UncheckedAccount<'info>,
    
    /// CHECK: Only used for CPI to gateway
    pub gateway_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Initiate cross-chain NFT transfer by burning local NFT and calling gateway
pub fn handler(
    ctx: Context<DepositAndCall>,
    transfer_id: String,
    destination_chain_id: u64,
    destination_recipient: [u8; 20], // Ethereum-style address
    revert_options: Option<gateway::RevertOptions>,
    bump: u8,
) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let universal_nft = &mut ctx.accounts.universal_nft;
    let cross_chain_transfer = &mut ctx.accounts.cross_chain_transfer;
    let clock = Clock::get()?;
    
    // Validate destination chain
    require!(
        destination_chain_id > 0 && destination_chain_id != SOLANA_CHAIN_ID,
        ErrorCode::InvalidChainId
    );
    
    // Lock the NFT for cross-chain transfer
    universal_nft.is_locked = true;
    universal_nft.lock_destination_chain = destination_chain_id.to_string();
    universal_nft.lock_recipient = hex::encode(destination_recipient);
    universal_nft.updated_at = clock.unix_timestamp;
    
    // Initialize cross-chain transfer state
    cross_chain_transfer.transfer_id = transfer_id.clone();
    cross_chain_transfer.nft_mint = ctx.accounts.nft_mint.key();
    cross_chain_transfer.source_owner = ctx.accounts.owner.key();
    cross_chain_transfer.destination_chain = destination_chain_id.to_string();
    cross_chain_transfer.destination_recipient = hex::encode(destination_recipient);
    cross_chain_transfer.status = TransferStatus::Initiated;
    cross_chain_transfer.initiated_at = clock.unix_timestamp;
    cross_chain_transfer.completed_at = None;
    cross_chain_transfer.bump = bump;
    
    // Burn the NFT since it's moving to another chain
    let cpi_accounts = Burn {
        mint: ctx.accounts.nft_mint.to_account_info(),
        from: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    burn(cpi_ctx, 1)?;
    
    // Prepare cross-chain message data
    let message_data = format!(
        "chain:{},token_id:{},uri:{},name:{},symbol:{}",
        universal_nft.original_chain,
        universal_nft.original_token_id,
        universal_nft.metadata_uri,
        "UniversalNFT", // Default name
        "UNFT" // Default symbol
    );
    
    // Call ZetaChain gateway to initiate cross-chain transfer
    let gateway_program = ctx.accounts.gateway_program.to_account_info();
    
    let cpi_accounts = gateway::cpi::accounts::DepositAndCall {
        signer: ctx.accounts.payer.to_account_info(),
        pda: ctx.accounts.gateway_pda.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_program = gateway_program;
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    gateway::cpi::deposit_and_call(
        cpi_ctx,
        global_config.cross_chain_fee,
        destination_recipient,
        destination_chain_id,
        message_data.as_bytes().to_vec(),
        revert_options,
    )?;
    
    msg!(
        "Cross-chain NFT transfer initiated via gateway: transfer_id={}, mint={}, destination_chain={}, recipient={:?}",
        transfer_id,
        ctx.accounts.nft_mint.key(),
        destination_chain_id,
        destination_recipient
    );
    
    Ok(())
}
