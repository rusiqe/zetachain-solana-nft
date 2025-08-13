use anchor_lang::prelude::*;

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(transfer_id: String)]
pub struct ConfirmCrossChainTransfer<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
        constraint = global_config.zetachain_gateway == zetachain_gateway.key() @ ErrorCode::GatewayNotConfigured
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(
        mut,
        seeds = [
            CROSS_CHAIN_TRANSFER_SEED,
            transfer_id.as_bytes(),
        ],
        bump = cross_chain_transfer.bump,
        constraint = cross_chain_transfer.status == TransferStatus::Initiated @ ErrorCode::InvalidTransferStatus,
    )]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    
    /// CHECK: This is the ZetaChain gateway - validated by global config
    pub zetachain_gateway: Signer<'info>,
}

pub fn handler(
    ctx: Context<ConfirmCrossChainTransfer>,
    transfer_id: String,
) -> Result<()> {
    let cross_chain_transfer = &mut ctx.accounts.cross_chain_transfer;
    
    // Update transfer status to confirmed
    cross_chain_transfer.status = TransferStatus::Confirmed;
    
    msg!(
        "CrossChainTransferConfirmed: transfer_id={}, mint={}, destination_chain={}, recipient={}",
        transfer_id,
        cross_chain_transfer.nft_mint,
        cross_chain_transfer.destination_chain,
        cross_chain_transfer.destination_recipient
    );
    
    Ok(())
}
