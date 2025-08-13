use anchor_lang::prelude::*;
use anchor_lang::solana_program::{self, sysvar::instructions::get_instruction_relative};
use anchor_spl::token::{self, Token};

use std::str::FromStr;

// Must match Anchor.toml
declare_id!("73ce2AD3AZpaGFNcdavnbKbhNGSmz3PNyv2GCDM3Yy3c");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        bump: u8,
        cross_chain_fee: Option<u64>,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.global_config;
        cfg.authority = ctx.accounts.authority.key();
        cfg.zetachain_gateway = ctx.accounts.zetachain_gateway.key();
        cfg.collection_authority = ctx.accounts.collection_authority.key();
        cfg.fee_recipient = ctx.accounts.fee_recipient.key();
        cfg.cross_chain_fee = cross_chain_fee.unwrap_or(0);
        cfg.bump = bump;
        Ok(())
    }

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
        // Minimal demo implementation: record metadata on the UniversalNft account.
        // A production implementation would mint an SPL NFT and set metadata via Metaplex.
        let acc = &mut ctx.accounts.universal_nft;
        acc.mint = ctx.accounts.mint.key();
        acc.owner = ctx.accounts.recipient.key();
        acc.original_chain = original_chain;
        acc.original_contract = original_contract;
        acc.original_token_id = original_token_id;
        acc.metadata_uri = uri;
        acc.is_locked = false;
        acc.lock_destination_chain = String::new();
        acc.lock_recipient = String::new();
        acc.created_at = Clock::get()?.unix_timestamp;
        acc.updated_at = acc.created_at;
        acc.bump = bump;
        emit!(NftMinted { mint: acc.mint, owner: acc.owner, name, symbol });
        Ok(())
    }

    pub fn initiate_cross_chain_transfer(
        ctx: Context<InitiateTransfer>,
        transfer_id: String,
        destination_chain: String,
        destination_recipient: String,
        _bump: u8,
    ) -> Result<()> {
        require!(ctx.accounts.global_config.cross_chain_fee <= ctx.accounts.payer.lamports(), ErrorCode::InsufficientFunds);
        let nft = &mut ctx.accounts.universal_nft;
        require!(!nft.is_locked, ErrorCode::NftLocked);
        // Lock the NFT for transfer
        nft.is_locked = true;
        nft.lock_destination_chain = destination_chain.clone();
        nft.lock_recipient = destination_recipient.clone();
        nft.updated_at = Clock::get()?.unix_timestamp;

        let xfer = &mut ctx.accounts.cross_chain_transfer;
        xfer.transfer_id = transfer_id.clone();
        xfer.nft_mint = ctx.accounts.nft_mint.key();
        xfer.source_owner = ctx.accounts.owner.key();
        xfer.destination_chain = destination_chain;
        xfer.destination_recipient = destination_recipient;
        xfer.status = TransferStatus::Initiated;
        xfer.initiated_at = Clock::get()?.unix_timestamp;
        xfer.completed_at = None;
        xfer.bump = 0;
        emit!(TransferInitiated { transfer_id });
        Ok(())
    }

    pub fn confirm_cross_chain_transfer(ctx: Context<ConfirmTransfer>, transfer_id: String) -> Result<()> {
        // In a real flow, this would be called by the ZetaChain gateway (EVM side confirmation),
        // but we keep it signer-gated for demo as per the existing IDL.
        let xfer = &mut ctx.accounts.cross_chain_transfer;
        require!(xfer.transfer_id == transfer_id, ErrorCode::TransferNotFound);
        xfer.status = TransferStatus::Confirmed;
        Ok(())
    }

    pub fn complete_cross_chain_transfer(ctx: Context<CompleteTransfer>, transfer_id: String) -> Result<()> {
        let xfer = &mut ctx.accounts.cross_chain_transfer;
        require!(xfer.transfer_id == transfer_id, ErrorCode::TransferNotFound);
        // For demo, mark completed and unlock NFT
        let nft = &mut ctx.accounts.universal_nft;
        nft.is_locked = false;
        nft.lock_destination_chain.clear();
        nft.lock_recipient.clear();
        nft.updated_at = Clock::get()?.unix_timestamp;

        xfer.status = TransferStatus::Completed;
        xfer.completed_at = Some(Clock::get()?.unix_timestamp);
        emit!(TransferCompleted { transfer_id });
        Ok(())
    }

    // GATEWAY-BASED DEMO INSTRUCTIONS

    // Outbound: user initiates a gateway call. For demo we only record the intent
    // and require that the provided gateway account matches configured one.
    pub fn deposit_and_call(
        ctx: Context<DepositAndCall>,
        transfer_id: String,
        destination_chain_id: u64,
        destination_recipient: [u8; 20],
        _revert_options: Option<RevertOptions>,
        _bump: u8,
    ) -> Result<()> {
        // Basic validation
        let cfg = &ctx.accounts.global_config;
        require_keys_eq!(cfg.zetachain_gateway, ctx.accounts.gateway_program.key(), ErrorCode::GatewayNotConfigured);

        // Record transfer intent and lock NFT
        let nft = &mut ctx.accounts.universal_nft;
        require!(!nft.is_locked, ErrorCode::NftLocked);
        nft.is_locked = true;
        nft.lock_destination_chain = destination_chain_id.to_string();
        nft.lock_recipient = hex_encode_20(destination_recipient);
        nft.updated_at = Clock::get()?.unix_timestamp;

        let xfer = &mut ctx.accounts.cross_chain_transfer;
        xfer.transfer_id = transfer_id.clone();
        xfer.nft_mint = ctx.accounts.nft_mint.key();
        xfer.source_owner = ctx.accounts.owner.key();
        xfer.destination_chain = nft.lock_destination_chain.clone();
        xfer.destination_recipient = nft.lock_recipient.clone();
        xfer.status = TransferStatus::Initiated;
        xfer.initiated_at = Clock::get()?.unix_timestamp;
        xfer.completed_at = None;
        xfer.bump = 0;

        emit!(GatewayDepositAndCall { transfer_id, destination_chain_id });
        Ok(())
    }

    // Inbound: called by the real gateway. We validate caller using instruction sysvar
    pub fn on_call(
        ctx: Context<OnCall>,
        _amount: u64,
        _sender: [u8; 20],
        data: Vec<u8>,
        bump: u8,
    ) -> Result<()> {
        validate_gateway_caller(&ctx.accounts.instruction_sysvar_account, &ctx.accounts.global_config)?;

        // Parse data (best-effort demo: look for uri field)
        let parsed = parse_kv_pairs(&data)?;
        let uri = parsed.get("uri").cloned().unwrap_or_default();
        let name = parsed.get("name").cloned().unwrap_or_else(|| "UniversalNFT".to_string());
        let symbol = parsed.get("symbol").cloned().unwrap_or_else(|| "UNFT".to_string());

        let nft = &mut ctx.accounts.universal_nft;
        nft.mint = ctx.accounts.mint.key();
        nft.owner = ctx.accounts.recipient.key();
        nft.metadata_uri = uri;
        nft.original_chain = parsed.get("chain").cloned().unwrap_or_default();
        nft.original_contract = parsed.get("contract").cloned().unwrap_or_default();
        nft.original_token_id = parsed.get("token_id").cloned().unwrap_or_default();
        nft.is_locked = false;
        nft.lock_destination_chain.clear();
        nft.lock_recipient.clear();
        nft.created_at = Clock::get()?.unix_timestamp;
        nft.updated_at = nft.created_at;
        nft.bump = bump;

        emit!(NftMinted { mint: nft.mint, owner: nft.owner, name, symbol });
        Ok(())
    }

    pub fn on_revert(
        ctx: Context<OnRevert>,
        _amount: u64,
        _sender: Pubkey,
        _data: Vec<u8>,
        transfer_id: String,
    ) -> Result<()> {
        validate_gateway_caller(&ctx.accounts.instruction_sysvar_account, &ctx.accounts.global_config)?;

        let nft = &mut ctx.accounts.universal_nft;
        nft.is_locked = false;
        nft.lock_destination_chain.clear();
        nft.lock_recipient.clear();
        nft.updated_at = Clock::get()?.unix_timestamp;

        let xfer = &mut ctx.accounts.cross_chain_transfer;
        require!(xfer.transfer_id == transfer_id, ErrorCode::TransferNotFound);
        xfer.status = TransferStatus::Failed;
        xfer.completed_at = Some(Clock::get()?.unix_timestamp);
        emit!(TransferReverted { transfer_id: xfer.transfer_id.clone() });
        Ok(())
    }
}

// Contexts
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, seeds = [b"global_config"], bump, space = 8 + GlobalConfig::INIT_SPACE)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: set into config
    pub zetachain_gateway: UncheckedAccount<'info>,
    /// CHECK: set into config
    pub collection_authority: UncheckedAccount<'info>,
    /// CHECK: set into config
    pub fee_recipient: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(init, payer = payer, seeds = [b"universal_nft", mint.key().as_ref()], bump, space = 8 + UniversalNft::INIT_SPACE)]
    pub universal_nft: Account<'info, UniversalNft>,
    /// CHECK: demo only
    pub mint: Signer<'info>,
    /// CHECK: demo only
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: demo only
    pub recipient: UncheckedAccount<'info>,
    pub collection_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: not used in demo
    pub associated_token_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitiateTransfer<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut, seeds = [b"universal_nft", nft_mint.key().as_ref()], bump = universal_nft.bump)]
    pub universal_nft: Account<'info, UniversalNft>,
    #[account(init_if_needed, payer = payer, seeds = [b"cross_chain_transfer", transfer_id.as_bytes()], bump, space = 8 + CrossChainTransfer::INIT_SPACE)]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    /// CHECK: demo only
    pub nft_mint: UncheckedAccount<'info>,
    /// CHECK: demo only
    #[account(mut)]
    pub owner_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: not used in demo
    pub zetachain_gateway: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfirmTransfer<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut, seeds = [b"cross_chain_transfer", transfer_id.as_bytes()], bump = cross_chain_transfer.bump)]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    /// CHECK: signer required by existing IDL
    pub zetachain_gateway: Signer<'info>,
}

#[derive(Accounts)]
pub struct CompleteTransfer<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut, seeds = [b"universal_nft", nft_mint.key().as_ref()], bump = universal_nft.bump)]
    pub universal_nft: Account<'info, UniversalNft>,
    #[account(mut, seeds = [b"cross_chain_transfer", transfer_id.as_bytes()], bump = cross_chain_transfer.bump)]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    /// CHECK: demo only
    #[account(mut)]
    pub nft_mint: UncheckedAccount<'info>,
    /// CHECK: demo only
    #[account(mut)]
    pub owner_token_account: UncheckedAccount<'info>,
    pub collection_authority: Signer<'info>,
    /// CHECK: signer per existing IDL
    pub zetachain_gateway: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DepositAndCall<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut, seeds = [b"universal_nft", nft_mint.key().as_ref()], bump = universal_nft.bump)]
    pub universal_nft: Account<'info, UniversalNft>,
    #[account(init_if_needed, payer = payer, seeds = [b"cross_chain_transfer", transfer_id.as_bytes()], bump, space = 8 + CrossChainTransfer::INIT_SPACE)]
    pub cross_chain_transfer: Account<'info, CrossChainTransfer>,
    /// CHECK: demo only
    pub nft_mint: UncheckedAccount<'info>,
    /// CHECK: demo only
    #[account(mut)]
    pub owner_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: gateway program id account for validation
    pub gateway_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnCall<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(init_if_needed, payer = payer, seeds = [b"universal_nft", mint.key().as_ref()], bump, space = 8 + UniversalNft::INIT_SPACE)]
    pub universal_nft: Account<'info, UniversalNft>,
    /// CHECK: demo only
    pub mint: UncheckedAccount<'info>,
    /// CHECK: demo only
    pub recipient: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: required for instruction introspection
    #[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OnRevert<'info> {
    #[account(seeds = [b"global_config"], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut, seeds = [b"universal_nft", mint.key().as_ref()], bump = universal_nft.bump)]
    pub universal_nft: Account<'info, UniversalNft>,
    /// CHECK: demo only
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: required for instruction introspection
    #[account(address = solana_program::sysvar::instructions::id())]
    pub instruction_sysvar_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

// Accounts
#[account]
pub struct GlobalConfig {
    pub authority: Pubkey,
    pub zetachain_gateway: Pubkey,
    pub collection_authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub cross_chain_fee: u64,
    pub bump: u8,
}

impl GlobalConfig { pub const INIT_SPACE: usize = 32 + 32 + 32 + 32 + 8 + 1; }

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

impl UniversalNft { pub const INIT_SPACE: usize = 32 + 32 + 4 + 32 + 4 + 64 + 4 + 200 + 1 + 4 + 32 + 4 + 64 + 8 + 8 + 1; }

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

impl CrossChainTransfer { pub const INIT_SPACE: usize = 4 + 64 + 32 + 32 + 4 + 32 + 4 + 64 + 1 + 8 + 1 + 8 + 1; }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TransferStatus { Initiated, Confirmed, Completed, Failed }

#[event]
pub struct NftMinted { pub mint: Pubkey, pub owner: Pubkey, pub name: String, pub symbol: String }
#[event]
pub struct TransferInitiated { pub transfer_id: String }
#[event]
pub struct TransferCompleted { pub transfer_id: String }
#[event]
pub struct TransferReverted { pub transfer_id: String }
#[event]
pub struct GatewayDepositAndCall { pub transfer_id: String, pub destination_chain_id: u64 }

// Types used for compatibility with guide
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct RevertOptions {
    pub revert_address: [u8; 32],
    pub call_on_revert: bool,
    pub revert_message: String,
}

// Utils
fn validate_gateway_caller(instr_sysvar: &UncheckedAccount, cfg: &Account<GlobalConfig>) -> Result<()> {
    let current_ix = get_instruction_relative(0, &instr_sysvar.to_account_info())
        .map_err(|_| error!(ErrorCode::Unauthorized))?;
    require!(current_ix.program_id == cfg.zetachain_gateway, ErrorCode::Unauthorized);
    Ok(())
}

fn parse_kv_pairs(data: &Vec<u8>) -> Result<std::collections::BTreeMap<String, String>> {
    let s = String::from_utf8(data.clone()).map_err(|_| error!(ErrorCode::InvalidMetadataUri))?;
    let mut map = std::collections::BTreeMap::new();
    for part in s.split(',') {
        let mut it = part.splitn(2, ':');
        if let (Some(k), Some(v)) = (it.next(), it.next()) {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

fn hex_encode_20(addr: [u8; 20]) -> String { format!("0x{}", hex::encode(addr)) }

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for cross-chain operation")] InsufficientFunds = 6000,
    #[msg("NFT is currently locked for cross-chain transfer")] NftLocked = 6001,
    #[msg("Invalid chain ID specified")] InvalidChainId = 6002,
    #[msg("Unauthorized operation")] Unauthorized = 6003,
    #[msg("Invalid metadata URI")] InvalidMetadataUri = 6004,
    #[msg("Cross-chain transfer already exists")] TransferAlreadyExists = 6005,
    #[msg("Cross-chain transfer not found")] TransferNotFound = 6006,
    #[msg("Invalid transfer status")] InvalidTransferStatus = 6007,
    #[msg("ZetaChain gateway not configured")] GatewayNotConfigured = 6008,
    #[msg("Compute budget exceeded")] ComputeBudgetExceeded = 6009,
    #[msg("Invalid destination address format")] InvalidDestinationAddress = 6010,
    #[msg("NFT mint failed")] MintFailed = 6011,
    #[msg("Invalid original chain data")] InvalidOriginalChain = 6012,
    #[msg("Transfer confirmation timeout")] TransferTimeout = 6013,
}
