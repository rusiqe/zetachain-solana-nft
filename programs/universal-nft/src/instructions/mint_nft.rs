use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(bump: u8, original_chain: String, original_contract: String, original_token_id: String)]
pub struct MintNft<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
        constraint = global_config.collection_authority == collection_authority.key() @ ErrorCode::Unauthorized
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(
        init,
        payer = payer,
        space = UniversalNft::INIT_SPACE,
        seeds = [
            UNIVERSAL_NFT_SEED,
            mint.key().as_ref(),
        ],
        bump
    )]
    pub universal_nft: Account<'info, UniversalNft>,
    
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_authority,
        mint::freeze_authority = collection_authority,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    // Metadata will be added in future iterations
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The recipient of the NFT
    /// CHECK: This can be any valid Solana address
    pub recipient: AccountInfo<'info>,
    
    /// Collection authority (must match global config)
    pub collection_authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<MintNft>,
    bump: u8,
    name: String,
    symbol: String,
    uri: String,
    original_chain: String,
    original_contract: String,
    original_token_id: String,
) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let mint = &ctx.accounts.mint;
    let universal_nft = &mut ctx.accounts.universal_nft;
    let clock = Clock::get()?;
    
    // Validate input parameters
    require!(!name.is_empty() && name.len() <= 32, ErrorCode::InvalidMetadataUri);
    require!(!symbol.is_empty() && symbol.len() <= 10, ErrorCode::InvalidMetadataUri);
    require!(!uri.is_empty() && uri.len() <= 200, ErrorCode::InvalidMetadataUri);
    require!(!original_chain.is_empty() && original_chain.len() <= 32, ErrorCode::InvalidOriginalChain);
    require!(!original_contract.is_empty() && original_contract.len() <= 64, ErrorCode::InvalidOriginalChain);
    require!(!original_token_id.is_empty() && original_token_id.len() <= 32, ErrorCode::InvalidOriginalChain);
    
    // Initialize universal NFT state
    universal_nft.mint = mint.key();
    universal_nft.owner = ctx.accounts.recipient.key();
    universal_nft.original_chain = original_chain;
    universal_nft.original_contract = original_contract;
    universal_nft.original_token_id = original_token_id;
    universal_nft.metadata_uri = uri.clone();
    universal_nft.is_locked = false;
    universal_nft.lock_destination_chain = String::new();
    universal_nft.lock_recipient = String::new();
    universal_nft.created_at = clock.unix_timestamp;
    universal_nft.updated_at = clock.unix_timestamp;
    universal_nft.bump = bump;
    
    // Store metadata in our program state (metadata can be added later via separate instruction)
    // For now, we store the metadata URI in the universal_nft account
    
    // Mint the NFT to the recipient
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.collection_authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    mint_to(cpi_ctx, 1)?;
    
    msg!(
        "Universal NFT minted: mint={}, recipient={}, original_chain={}, original_token_id={}",
        mint.key(),
        ctx.accounts.recipient.key(),
        universal_nft.original_chain,
        universal_nft.original_token_id
    );
    
    Ok(())
}
