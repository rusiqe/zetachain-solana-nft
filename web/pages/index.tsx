import Head from 'next/head'
import { useEffect, useMemo, useState } from 'react'
import { AnchorProvider, Program, Idl, BN } from '@coral-xyz/anchor'
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js'
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react'
import { WalletModalProvider, WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { PhantomWalletAdapter, SolflareWalletAdapter, SolletWalletAdapter } from '@solana/wallet-adapter-wallets'

import idl from '../idl/universal_nft.json'

const PROGRAM_ID = new PublicKey((idl as any).address)

export default function Home() {
  const [signature, setSignature] = useState<string>('')
  const [status, setStatus] = useState<string>('Idle')

  const network = WalletAdapterNetwork.Devnet
  const endpoint = useMemo(() => clusterApiUrl(network), [network])
  const wallets = useMemo(() => [new PhantomWalletAdapter(), new SolflareWalletAdapter(), new SolletWalletAdapter() as any], [])

  return (
    
      
        
        
        
        
          ZetaChain Universal NFT Demo
        
        
          
            
          
        
        
          
            Program ID: {PROGRAM_ID.toBase58()}
          
          
            Status: {status}
            {signature && (
              
                Last TX: {signature}
              
            )}
          
          
            Mint Demo NFT
          
        
      
  )
}

