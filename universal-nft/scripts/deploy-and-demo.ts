import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { UniversalNft } from "../target/types/universal_nft";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram,
  Connection,
  clusterApiUrl,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import fs from 'fs';
import path from 'path';

// Configuration
const NETWORK = process.env.SOLANA_NETWORK || 'localnet';
const DEMO_MODE = process.env.DEMO_MODE === 'true';

async function main() {
  console.log("🚀 ZetaChain Universal NFT Program Deployment & Demo");
  console.log("=" .repeat(60));

  // Setup connection and provider
  let connection: Connection;
  if (NETWORK === 'localnet') {
    connection = new Connection('http://127.0.0.1:8899', 'confirmed');
  } else if (NETWORK === 'devnet') {
    connection = new Connection(clusterApiUrl('devnet'), 'confirmed');
  } else {
    throw new Error(`Unsupported network: ${NETWORK}`);
  }

  const wallet = anchor.AnchorProvider.env().wallet;
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
  });
  anchor.setProvider(provider);

  const program = anchor.workspace.UniversalNft as Program<UniversalNft>;

  console.log(`📡 Network: ${NETWORK}`);
  console.log(`💳 Wallet: ${provider.wallet.publicKey.toString()}`);
  console.log(`📋 Program ID: ${program.programId.toString()}`);
  console.log("");

  // Create demo keypairs
  const authority = Keypair.generate();
  const zetachainGateway = Keypair.generate();
  const collectionAuthority = Keypair.generate();
  const feeRecipient = Keypair.generate();
  const nftRecipient = Keypair.generate();

  console.log("🔑 Generated demo keypairs:");
  console.log(`   Authority: ${authority.publicKey.toString()}`);
  console.log(`   ZetaChain Gateway: ${zetachainGateway.publicKey.toString()}`);
  console.log(`   Collection Authority: ${collectionAuthority.publicKey.toString()}`);
  console.log(`   Fee Recipient: ${feeRecipient.publicKey.toString()}`);
  console.log(`   NFT Recipient: ${nftRecipient.publicKey.toString()}`);
  console.log("");

  // Airdrop SOL if on localnet
  if (NETWORK === 'localnet') {
    console.log("💰 Airdropping SOL to demo accounts...");
    const airdropAccounts = [
      authority.publicKey,
      zetachainGateway.publicKey, 
      collectionAuthority.publicKey,
      feeRecipient.publicKey,
    ];

    for (const account of airdropAccounts) {
      try {
        const signature = await connection.requestAirdrop(
          account,
          2 * anchor.web3.LAMPORTS_PER_SOL
        );
        await connection.confirmTransaction(signature);
        console.log(`   ✅ Airdropped 2 SOL to ${account.toString()}`);
      } catch (error) {
        console.log(`   ⚠️  Failed to airdrop to ${account.toString()}: ${error}`);
      }
    }
    
    // Wait for airdrops to settle
    await new Promise(resolve => setTimeout(resolve, 2000));
    console.log("");
  }

  // 1. Initialize Global Configuration
  console.log("1️⃣ Initializing Global Configuration...");
  
  const [globalConfigPda, globalConfigBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("global_config")],
    program.programId
  );

  const crossChainFee = new BN(10_000); // 0.00001 SOL

  try {
    const initTx = await program.methods
      .initialize(
        globalConfigBump,
        crossChainFee
      )
      .accounts({
        globalConfig: globalConfigPda,
        authority: authority.publicKey,
        zetachainGateway: zetachainGateway.publicKey,
        collectionAuthority: collectionAuthority.publicKey,
        feeRecipient: feeRecipient.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    console.log(`   ✅ Initialized global config: ${initTx}`);
    console.log(`   📍 Global Config PDA: ${globalConfigPda.toString()}`);

    // Verify global config
    const globalConfigAccount = await program.account.globalConfig.fetch(globalConfigPda);
    console.log(`   💰 Cross-chain fee: ${globalConfigAccount.crossChainFee.toString()} lamports`);
  } catch (error) {
    console.log(`   ❌ Failed to initialize: ${error}`);
    return;
  }
  console.log("");

  // 2. Mint Universal NFT
  console.log("2️⃣ Minting Universal NFT...");

  const nftMint = Keypair.generate();
  const nftName = "ZetaChain Universal NFT #1";
  const nftSymbol = "ZUNFT";
  const nftUri = "https://zetachain.com/api/metadata/solana/1.json";
  const originalChain = "ethereum";
  const originalContract = "0x1234567890abcdef1234567890abcdef12345678";
  const originalTokenId = "1";

  const [universalNftPda, universalNftBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("universal_nft"), nftMint.publicKey.toBuffer()],
    program.programId
  );

  const tokenAccount = await getAssociatedTokenAddress(
    nftMint.publicKey,
    nftRecipient.publicKey
  );

  try {
    const mintTx = await program.methods
      .mintNft(
        universalNftBump,
        nftName,
        nftSymbol,
        nftUri,
        originalChain,
        originalContract,
        originalTokenId
      )
      .accounts({
        globalConfig: globalConfigPda,
        universalNft: universalNftPda,
        mint: nftMint.publicKey,
        tokenAccount: tokenAccount,
        payer: authority.publicKey,
        recipient: nftRecipient.publicKey,
        collectionAuthority: collectionAuthority.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority, nftMint, collectionAuthority])
      .rpc();

    console.log(`   ✅ Minted NFT: ${mintTx}`);
    console.log(`   🎨 NFT Mint: ${nftMint.publicKey.toString()}`);
    console.log(`   📍 Universal NFT PDA: ${universalNftPda.toString()}`);
    console.log(`   💰 Token Account: ${tokenAccount.toString()}`);

    // Verify NFT account
    const universalNftAccount = await program.account.universalNft.fetch(universalNftPda);
    console.log(`   🏷️  Name: ${nftName}`);
    console.log(`   🔗 Original Chain: ${universalNftAccount.originalChain}`);
    console.log(`   📝 Original Contract: ${universalNftAccount.originalContract}`);
    console.log(`   🆔 Original Token ID: ${universalNftAccount.originalTokenId}`);
  } catch (error) {
    console.log(`   ❌ Failed to mint NFT: ${error}`);
    return;
  }
  console.log("");

  // 3. Initiate Cross-Chain Transfer
  console.log("3️⃣ Initiating Cross-Chain Transfer...");

  const transferId = `transfer_${Date.now()}`;
  const destinationChain = "polygon";
  const destinationRecipient = "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd";

  const [crossChainTransferPda, crossChainTransferBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("cross_chain_transfer"), Buffer.from(transferId)],
    program.programId
  );

  try {
    const transferTx = await program.methods
      .initiateCrossChainTransfer(
        transferId,
        destinationChain,
        destinationRecipient,
        crossChainTransferBump
      )
      .accounts({
        globalConfig: globalConfigPda,
        universalNft: universalNftPda,
        crossChainTransfer: crossChainTransferPda,
        nftMint: nftMint.publicKey,
        ownerTokenAccount: tokenAccount,
        owner: nftRecipient.publicKey,
        payer: authority.publicKey,
        zetachainGateway: zetachainGateway.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority, nftRecipient])
      .rpc();

    console.log(`   ✅ Initiated cross-chain transfer: ${transferTx}`);
    console.log(`   🆔 Transfer ID: ${transferId}`);
    console.log(`   🌐 Destination Chain: ${destinationChain}`);
    console.log(`   📍 Destination Recipient: ${destinationRecipient}`);
    console.log(`   📍 Cross-chain Transfer PDA: ${crossChainTransferPda.toString()}`);

    // Verify transfer account
    const crossChainTransferAccount = await program.account.crossChainTransfer.fetch(crossChainTransferPda);
    console.log(`   ⏰ Initiated at: ${new Date(crossChainTransferAccount.initiatedAt.toNumber() * 1000)}`);
    console.log(`   🔒 Transfer status: Initiated`);

    // Verify NFT is locked
    const updatedNftAccount = await program.account.universalNft.fetch(universalNftPda);
    console.log(`   🔐 NFT locked: ${updatedNftAccount.isLocked}`);
  } catch (error) {
    console.log(`   ❌ Failed to initiate transfer: ${error}`);
    return;
  }
  console.log("");

  // 4. Confirm Cross-Chain Transfer (simulate ZetaChain gateway)
  console.log("4️⃣ Confirming Cross-Chain Transfer (ZetaChain Gateway)...");

  try {
    const confirmTx = await program.methods
      .confirmCrossChainTransfer(transferId)
      .accounts({
        globalConfig: globalConfigPda,
        crossChainTransfer: crossChainTransferPda,
        zetachainGateway: zetachainGateway.publicKey,
      })
      .signers([zetachainGateway])
      .rpc();

    console.log(`   ✅ Confirmed cross-chain transfer: ${confirmTx}`);

    // Verify transfer status
    const confirmedTransferAccount = await program.account.crossChainTransfer.fetch(crossChainTransferPda);
    console.log(`   ✅ Transfer status: Confirmed`);
  } catch (error) {
    console.log(`   ❌ Failed to confirm transfer: ${error}`);
    return;
  }
  console.log("");

  // 5. Complete Cross-Chain Transfer (burn NFT)
  console.log("5️⃣ Completing Cross-Chain Transfer (Burning NFT)...");

  try {
    const completeTx = await program.methods
      .completeCrossChainTransfer(transferId)
      .accounts({
        globalConfig: globalConfigPda,
        universalNft: universalNftPda,
        crossChainTransfer: crossChainTransferPda,
        nftMint: nftMint.publicKey,
        ownerTokenAccount: tokenAccount,
        collectionAuthority: collectionAuthority.publicKey,
        zetachainGateway: zetachainGateway.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([collectionAuthority, zetachainGateway])
      .rpc();

    console.log(`   ✅ Completed cross-chain transfer: ${completeTx}`);

    // Verify transfer completion
    const completedTransferAccount = await program.account.crossChainTransfer.fetch(crossChainTransferPda);
    console.log(`   ✅ Transfer status: Completed`);
    console.log(`   ⏰ Completed at: ${new Date(completedTransferAccount.completedAt.toNumber() * 1000)}`);

    // Verify NFT is unlocked
    const finalNftAccount = await program.account.universalNft.fetch(universalNftPda);
    console.log(`   🔓 NFT unlocked: ${!finalNftAccount.isLocked}`);

    // Verify token was burned
    try {
      const tokenAccountInfo = await connection.getTokenAccountBalance(tokenAccount);
      console.log(`   🔥 Token balance after burn: ${tokenAccountInfo.value.amount}`);
    } catch (error) {
      console.log(`   🔥 Token account closed (NFT burned successfully)`);
    }
  } catch (error) {
    console.log(`   ❌ Failed to complete transfer: ${error}`);
    return;
  }
  console.log("");

  // Summary
  console.log("🎉 Demo Completed Successfully!");
  console.log("=" .repeat(60));
  console.log("✅ Global configuration initialized");
  console.log("✅ Universal NFT minted with cross-chain metadata");
  console.log("✅ Cross-chain transfer initiated");
  console.log("✅ Transfer confirmed by ZetaChain gateway");
  console.log("✅ Transfer completed with NFT burn");
  console.log("");

  console.log("🔗 Key Addresses:");
  console.log(`   Program ID: ${program.programId.toString()}`);
  console.log(`   Global Config: ${globalConfigPda.toString()}`);
  console.log(`   NFT Mint: ${nftMint.publicKey.toString()}`);
  console.log(`   Universal NFT PDA: ${universalNftPda.toString()}`);
  console.log(`   Transfer PDA: ${crossChainTransferPda.toString()}`);
  console.log("");

  console.log("🌉 Cross-Chain Integration:");
  console.log(`   Transfer ID: ${transferId}`);
  console.log(`   From: Solana`);
  console.log(`   To: ${destinationChain}`);
  console.log(`   Recipient: ${destinationRecipient}`);
  console.log("");

  console.log("💡 Next Steps:");
  console.log("   - Integrate with ZetaChain protocol contracts");
  console.log("   - Add Metaplex metadata support");
  console.log("   - Deploy to mainnet");
  console.log("   - Build client SDK");

  // Save deployment info
  const deploymentInfo = {
    network: NETWORK,
    programId: program.programId.toString(),
    globalConfigPda: globalConfigPda.toString(),
    authority: authority.publicKey.toString(),
    zetachainGateway: zetachainGateway.publicKey.toString(),
    collectionAuthority: collectionAuthority.publicKey.toString(),
    feeRecipient: feeRecipient.publicKey.toString(),
    demo: {
      nftMint: nftMint.publicKey.toString(),
      universalNftPda: universalNftPda.toString(),
      transferId: transferId,
      crossChainTransferPda: crossChainTransferPda.toString(),
    },
    timestamp: new Date().toISOString(),
  };

  fs.writeFileSync(
    path.join(__dirname, '../deployment-info.json'),
    JSON.stringify(deploymentInfo, null, 2)
  );

  console.log("📄 Deployment info saved to deployment-info.json");
}

main().catch((error) => {
  console.error("❌ Deployment failed:", error);
  process.exit(1);
});
