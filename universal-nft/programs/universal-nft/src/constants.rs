use anchor_lang::prelude::*;

/// Program constants
#[constant]
pub const SEED: &str = "universal_nft";

/// Maximum compute units for cross-chain operations
pub const MAX_COMPUTE_UNITS: u32 = 400_000;

/// Minimum SOL for rent exemption
pub const MIN_RENT_EXEMPTION: u64 = 2_039_280; // ~0.002 SOL

/// ZetaChain specific constants
pub const ZETACHAIN_CHAIN_ID: u64 = 7000;
pub const ETHEREUM_CHAIN_ID: u64 = 1;
pub const BSC_CHAIN_ID: u64 = 56;
pub const POLYGON_CHAIN_ID: u64 = 137;
pub const SOLANA_CHAIN_ID: u64 = 900; // Solana testnet chain ID

/// Cross-chain message types
pub const MSG_TYPE_MINT: u8 = 1;
pub const MSG_TYPE_TRANSFER: u8 = 2;
pub const MSG_TYPE_BURN: u8 = 3;
pub const MSG_TYPE_LOCK: u8 = 4;
pub const MSG_TYPE_UNLOCK: u8 = 5;

/// Default fees (in lamports)
pub const DEFAULT_CROSS_CHAIN_FEE: u64 = 10_000; // 0.00001 SOL
pub const DEFAULT_MINT_FEE: u64 = 5_000; // 0.000005 SOL
