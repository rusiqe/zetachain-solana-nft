use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = GlobalConfig::INIT_SPACE,
        seeds = [GLOBAL_CONFIG_SEED],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: This is the ZetaChain gateway address
    pub zetachain_gateway: AccountInfo<'info>,
    
    /// CHECK: This is the collection authority
    pub collection_authority: AccountInfo<'info>,
    
    /// CHECK: This is the fee recipient
    pub fee_recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    bump: u8,
    cross_chain_fee: Option<u64>,
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    
    // Validate that the ZetaChain gateway is provided
    require!(
        ctx.accounts.zetachain_gateway.key() != Pubkey::default(),
        ErrorCode::GatewayNotConfigured
    );
    
    global_config.authority = ctx.accounts.authority.key();
    global_config.zetachain_gateway = ctx.accounts.zetachain_gateway.key();
    global_config.collection_authority = ctx.accounts.collection_authority.key();
    global_config.fee_recipient = ctx.accounts.fee_recipient.key();
    global_config.cross_chain_fee = cross_chain_fee.unwrap_or(DEFAULT_CROSS_CHAIN_FEE);
    global_config.bump = bump;
    
    msg!(
        "Universal NFT program initialized with authority: {}, gateway: {}",
        global_config.authority,
        global_config.zetachain_gateway
    );
    
    Ok(())
}
