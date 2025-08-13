use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for cross-chain operation")]
    InsufficientFunds,
    
    #[msg("NFT is currently locked for cross-chain transfer")]
    NftLocked,
    
    #[msg("Invalid chain ID specified")]
    InvalidChainId,
    
    #[msg("Unauthorized operation")]
    Unauthorized,
    
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,
    
    #[msg("Cross-chain transfer already exists")]
    TransferAlreadyExists,
    
    #[msg("Cross-chain transfer not found")]
    TransferNotFound,
    
    #[msg("Invalid transfer status")]
    InvalidTransferStatus,
    
    #[msg("ZetaChain gateway not configured")]
    GatewayNotConfigured,
    
    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded,
    
    #[msg("Invalid destination address format")]
    InvalidDestinationAddress,
    
    #[msg("NFT mint failed")]
    MintFailed,
    
    #[msg("Invalid original chain data")]
    InvalidOriginalChain,
    
    #[msg("Transfer confirmation timeout")]
    TransferTimeout,
}
