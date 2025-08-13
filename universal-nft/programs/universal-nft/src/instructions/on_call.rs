use anchor_lang::prelude::*;
use anchor_lang::solana_program::{sysvar, sysvar::instructions::get_instruction_relative};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

use crate::state::*;
use crate::error::ErrorCode;
use crate::constants::*;

#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(
        seeds = [GLOBAL_CONFIG_SEED],
        bump = global_config.bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    #[account(
        init_if_needed,
        payer = payer,
        space = UniversalNft::INIT_SPACE,
        seeds = [
            UNIVERSAL_NFT_SEED,
            mint.key().as_ref(),
        ],
        bump
    )]
    pub universal_nft: Account<'info, UniversalNft>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is the ZetaChain gateway PDA
    pub gateway_pda: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// CHECK: Recipient address for NFT
    pub recipient: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: Instruction sysvar for gateway validation
    #[account(address = sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
}

/// Called by ZetaChain gateway when receiving cross-chain NFT transfer
pub fn handler(
    ctx: Context<OnCall>,
    amount: u64,
    sender: [u8; 20], // Ethereum-style address
    data: Vec<u8>,
    bump: u8,
) -> Result<()> {
    // Validate that caller is the ZetaChain gateway
    let current_ix = get_instruction_relative(
        0,
        &ctx.accounts.instruction_sysvar_account.to_account_info(),
    )
    .unwrap();
    
    msg!(
        "on_call invoked by: {}, expected gateway: {}",
        current_ix.program_id,
        ctx.accounts.global_config.zetachain_gateway
    );
    
    require!(
        current_ix.program_id == ctx.accounts.global_config.zetachain_gateway,
        ErrorCode::Unauthorized
    );
    
    // Parse the cross-chain NFT data
    let nft_data = parse_cross_chain_nft_data(&data)?;
    
    let universal_nft = &mut ctx.accounts.universal_nft;
    let clock = Clock::get()?;
    
    // Initialize or update the universal NFT with cross-chain data
    universal_nft.mint = ctx.accounts.mint.key();
    universal_nft.owner = ctx.accounts.recipient.key();
    universal_nft.original_chain = nft_data.original_chain;
    universal_nft.original_contract = hex::encode(sender); // Convert sender to hex string
    universal_nft.original_token_id = nft_data.token_id;
    universal_nft.metadata_uri = nft_data.metadata_uri;
    universal_nft.is_locked = false;
    universal_nft.lock_destination_chain = String::new();
    universal_nft.lock_recipient = String::new();
    universal_nft.created_at = clock.unix_timestamp;
    universal_nft.updated_at = clock.unix_timestamp;
    universal_nft.bump = bump;
    
    // Mint the NFT to the recipient
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.token_account.to_account_info(),
        authority: ctx.accounts.global_config.to_account_info(),
    };
    
    let seeds = &[
        GLOBAL_CONFIG_SEED,
        &[ctx.accounts.global_config.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    mint_to(cpi_ctx, 1)?;
    
    msg!(
        "Cross-chain NFT minted: mint={}, sender={:?}, recipient={}, token_id={}",
        ctx.accounts.mint.key(),
        sender,
        ctx.accounts.recipient.key(),
        universal_nft.original_token_id
    );
    
    Ok(())
}

#[derive(Debug)]
pub struct CrossChainNftData {
    pub original_chain: String,
    pub token_id: String,
    pub metadata_uri: String,
    pub name: String,
    pub symbol: String,
}

/// Parse cross-chain NFT data from the message payload
fn parse_cross_chain_nft_data(data: &[u8]) -> Result<CrossChainNftData> {
    // In a real implementation, this would parse a structured format like JSON or protobuf
    // For this example, we'll assume a simple string format
    let message = String::from_utf8(data.to_vec())
        .map_err(|_| ErrorCode::InvalidOriginalChain)?;
    
    // Expected format: "chain:ethereum,token_id:123,uri:https://...,name:MyNFT,symbol:MNFT"
    let parts: Vec<&str> = message.split(',').collect();
    let mut nft_data = CrossChainNftData {
        original_chain: String::new(),
        token_id: String::new(),
        metadata_uri: String::new(),
        name: String::new(),
        symbol: String::new(),
    };
    
    for part in parts {
        let kv: Vec<&str> = part.split(':').collect();
        if kv.len() == 2 {
            match kv[0] {
                "chain" => nft_data.original_chain = kv[1].to_string(),
                "token_id" => nft_data.token_id = kv[1].to_string(),
                "uri" => nft_data.metadata_uri = kv[1].to_string(),
                "name" => nft_data.name = kv[1].to_string(),
                "symbol" => nft_data.symbol = kv[1].to_string(),
                _ => {}
            }
        }
    }
    
    // Validate required fields
    require!(!nft_data.original_chain.is_empty(), ErrorCode::InvalidOriginalChain);
    require!(!nft_data.token_id.is_empty(), ErrorCode::InvalidOriginalChain);
    
    Ok(nft_data)
}
