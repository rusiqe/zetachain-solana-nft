import { useCallback, useMemo, useState } from 'react'
import { clusterApiUrl, Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js'
import idl from '../idl/universal_nft.json'
import * as anchor from '@coral-xyz/anchor'
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'
import { ConnectionProvider, WalletProvider, useAnchorWallet } from '@solana/wallet-adapter-react'
import { WalletModalProvider, WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { PhantomWalletAdapter, SolflareWalletAdapter } from '@solana/wallet-adapter-wallets'
import { getAssociatedTokenAddress, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token'

const PROGRAM_ID = new PublicKey((idl as any).address)

function AppInner() {
  const wallet = useAnchorWallet()
  const [status, setStatus] = useState<string>('Ready')
  const [logs, setLogs] = useState<string>('')
  const connection = useMemo(() => new Connection('http://127.0.0.1:8899', 'confirmed'), [])

  const provider = useMemo(() => (wallet ? new anchor.AnchorProvider(connection, wallet, { commitment: 'confirmed' }) : null), [connection, wallet])
  const program = useMemo(() => (provider ? new anchor.Program(idl as any, PROGRAM_ID, provider) : null), [provider])

  const [gateway, setGateway] = useState<string>('11111111111111111111111111111111')
  const [fee, setFee] = useState<string>('10000')

  const [mintName, setMintName] = useState('ZUNFT')
  const [mintSymbol, setMintSymbol] = useState('ZUNFT')
  const [mintUri, setMintUri] = useState('https://example.com/1.json')
  const [originalChain, setOriginalChain] = useState('solana')
  const [originalContract, setOriginalContract] = useState('native')
  const [originalTokenId, setOriginalTokenId] = useState('1')

  const [lastMint, setLastMint] = useState<PublicKey | null>(null)

  const handleInitialize = useCallback(async () => {
    if (!program || !provider || !wallet?.publicKey) return
    setStatus('Initializing...')
    try {
      const [globalConfigPda, bump] = PublicKey.findProgramAddressSync([Buffer.from('global_config')], PROGRAM_ID)
      const tx = await program.methods
        .initialize(bump, new anchor.BN(parseInt(fee)))
        .accounts({
          globalConfig: globalConfigPda,
          authority: wallet.publicKey,
          zetachainGateway: new PublicKey(gateway),
          collectionAuthority: wallet.publicKey,
          feeRecipient: wallet.publicKey,
          systemProgram: SystemProgram.programId
        })
        .rpc()
      setStatus('Initialized ✅')
      setLogs(`Initialize tx: ${tx}`)
    } catch (e: any) {
      setStatus('Init failed')
      setLogs(e.message || String(e))
    }
  }, [program, provider, wallet, gateway, fee])

  const handleMint = useCallback(async () => {
    if (!program || !provider || !wallet?.publicKey) return
    setStatus('Minting...')
    try {
      const mint = Keypair.generate()
      const [universalNftPda, bump] = PublicKey.findProgramAddressSync([Buffer.from('universal_nft'), mint.publicKey.toBuffer()], PROGRAM_ID)
      const tokenAccount = await getAssociatedTokenAddress(mint.publicKey, wallet.publicKey)

      const tx = await program.methods
        .mintNft(bump, mintName, mintSymbol, mintUri, originalChain, originalContract, originalTokenId)
        .accounts({
          globalConfig: PublicKey.findProgramAddressSync([Buffer.from('global_config')], PROGRAM_ID)[0],
          universalNft: universalNftPda,
          mint: mint.publicKey,
          tokenAccount,
          payer: wallet.publicKey,
          recipient: wallet.publicKey,
          collectionAuthority: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        })
        .signers([mint])
        .rpc()

      setLastMint(mint.publicKey)
      setStatus('Minted ✅')
      setLogs(`Mint tx: ${tx}\nMint: ${mint.publicKey.toBase58()}`)
    } catch (e: any) {
      setStatus('Mint failed')
      setLogs(e.message || String(e))
    }
  }, [program, provider, wallet, mintName, mintSymbol, mintUri, originalChain, originalContract, originalTokenId])

  const handleInitiate = useCallback(async () => {
    if (!program || !provider || !wallet?.publicKey || !lastMint) return
    setStatus('Initiating cross-chain transfer...')
    try {
      const transferId = `transfer_${Date.now()}`
      const [universalNftPda] = PublicKey.findProgramAddressSync([Buffer.from('universal_nft'), lastMint.toBuffer()], PROGRAM_ID)
      const [crossChainTransferPda, crossBump] = PublicKey.findProgramAddressSync([Buffer.from('cross_chain_transfer'), Buffer.from(transferId)], PROGRAM_ID)
      const ownerTokenAccount = await getAssociatedTokenAddress(lastMint, wallet.publicKey)

      const tx = await program.methods
        .initiateCrossChainTransfer(transferId, 'polygon', '0xabc', crossBump)
        .accounts({
          globalConfig: PublicKey.findProgramAddressSync([Buffer.from('global_config')], PROGRAM_ID)[0],
          universalNft: universalNftPda,
          crossChainTransfer: crossChainTransferPda,
          nftMint: lastMint,
          ownerTokenAccount,
          owner: wallet.publicKey,
          payer: wallet.publicKey,
          zetachainGateway: PublicKey.default,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
        })
        .rpc()

      setStatus('Transfer initiated ✅')
      setLogs(`Initiate tx: ${tx}\nTransfer ID: ${transferId}`)
    } catch (e: any) {
      setStatus('Initiate failed')
      setLogs(e.message || String(e))
    }
  }, [program, provider, wallet, lastMint])

  return (
    <div style={{ fontFamily: 'sans-serif', maxWidth: 900, margin: '40px auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <h1 style={{ margin: 0 }}>ZetaChain Universal NFT Demo</h1>
        <WalletMultiButton />
      </div>
      <p>Program ID: <code>{PROGRAM_ID.toBase58()}</code></p>

      <section style={{ border: '1px solid #ddd', padding: 16, borderRadius: 8, marginBottom: 16 }}>
        <h3>1) Initialize</h3>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input style={{ flex: 1 }} value={gateway} onChange={e => setGateway(e.target.value)} placeholder="Gateway Program ID" />
          <input style={{ width: 180 }} value={fee} onChange={e => setFee(e.target.value)} placeholder="Fee (lamports)" />
          <button onClick={handleInitialize} disabled={!wallet}>Initialize</button>
        </div>
      </section>

      <section style={{ border: '1px solid #ddd', padding: 16, borderRadius: 8, marginBottom: 16 }}>
        <h3>2) Mint NFT</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 8 }}>
          <input value={mintName} onChange={e => setMintName(e.target.value)} placeholder="Name" />
          <input value={mintSymbol} onChange={e => setMintSymbol(e.target.value)} placeholder="Symbol" />
          <input value={mintUri} onChange={e => setMintUri(e.target.value)} placeholder="URI" />
          <input value={originalChain} onChange={e => setOriginalChain(e.target.value)} placeholder="Original Chain" />
          <input value={originalContract} onChange={e => setOriginalContract(e.target.value)} placeholder="Original Contract" />
          <input value={originalTokenId} onChange={e => setOriginalTokenId(e.target.value)} placeholder="Original Token ID" />
        </div>
        <div style={{ marginTop: 8 }}>
          <button onClick={handleMint} disabled={!wallet}>Mint</button>
        </div>
      </section>

      <section style={{ border: '1px solid #ddd', padding: 16, borderRadius: 8, marginBottom: 16 }}>
        <h3>3) Initiate Cross-Chain Transfer (Demo)</h3>
        <button onClick={handleInitiate} disabled={!wallet || !lastMint}>Initiate</button>
      </section>

      <section style={{ border: '1px solid #ddd', padding: 16, borderRadius: 8 }}>
        <h3>Status</h3>
        <p>{status}</p>
        <pre style={{ whiteSpace: 'pre-wrap' }}>{logs}</pre>
      </section>
    </div>
  )
}

export default function Home() {
  const endpoint = 'http://127.0.0.1:8899'
  const wallets = useMemo(() => [new PhantomWalletAdapter(), new SolflareWalletAdapter()], [])

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <AppInner />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  )
}

