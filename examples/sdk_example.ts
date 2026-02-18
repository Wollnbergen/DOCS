/**
 * Sultan L1 SDK Example - TypeScript/JavaScript
 * 
 * This example demonstrates how to interact with Sultan L1 blockchain.
 * 
 * ## Installation
 * ```bash
 * npm install @noble/ed25519 bech32 fast-json-stable-stringify
 * ```
 * 
 * ## Usage
 * ```bash
 * npx ts-node sdk_example.ts
 * ```
 */

import * as ed25519 from '@noble/ed25519';
import { bech32 } from 'bech32';
import stringify from 'fast-json-stable-stringify';

const RPC_URL = 'https://rpc.sltn.io';

// ============================================================================
// HELPERS
// ============================================================================

/**
 * Convert bytes to hex string (required for signatures and public keys)
 */
function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Convert hex string to bytes
 */
function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  return bytes;
}

// ============================================================================
// WALLET
// ============================================================================

interface Wallet {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
  address: string;
}

/**
 * Generate a new random wallet
 */
async function createWallet(): Promise<Wallet> {
  const privateKey = ed25519.utils.randomPrivateKey();
  const publicKey = await ed25519.getPublicKeyAsync(privateKey);
  
  // Derive address: SHA256(pubkey)[0:20] -> bech32("sultan")
  const hash = await crypto.subtle.digest('SHA-256', publicKey);
  const addressBytes = new Uint8Array(hash).slice(0, 20);
  const words = bech32.toWords(addressBytes);
  const address = bech32.encode('sultan', words);
  
  return { privateKey, publicKey, address };
}

/**
 * Import wallet from private key hex
 */
async function importWallet(privateKeyHex: string): Promise<Wallet> {
  const privateKey = hexToBytes(privateKeyHex);
  const publicKey = await ed25519.getPublicKeyAsync(privateKey);
  
  const hash = await crypto.subtle.digest('SHA-256', publicKey);
  const addressBytes = new Uint8Array(hash).slice(0, 20);
  const words = bech32.toWords(addressBytes);
  const address = bech32.encode('sultan', words);
  
  return { privateKey, publicKey, address };
}

// ============================================================================
// API TYPES
// ============================================================================

interface Balance {
  address: string;
  balance: number;
  nonce: number;
}

interface Status {
  node_id: string;
  block_height: number;
  validators: number;
  uptime_seconds: number;
  version: string;
  shard_count: number;
  tps_capacity: number;
}

interface TransactionResult {
  hash: string;
  from: string;
  to: string;
  amount: number;
  status: string;
  block_height?: number;
}

// ============================================================================
// SDK FUNCTIONS
// ============================================================================

/**
 * Get network status
 */
async function getStatus(): Promise<Status> {
  const res = await fetch(`${RPC_URL}/status`);
  return res.json();
}

/**
 * Get balance for an address
 */
async function getBalance(address: string): Promise<Balance> {
  const res = await fetch(`${RPC_URL}/balance/${address}`);
  return res.json();
}

/**
 * Get balance in human-readable SLTN
 */
async function getBalanceSltn(address: string): Promise<number> {
  const balance = await getBalance(address);
  return balance.balance / 1e9;
}

/**
 * Send SLTN tokens
 * 
 * CRITICAL: This function demonstrates proper signature format:
 * - Keys must be alphabetically sorted (use fast-json-stable-stringify)
 * - Amount must be a STRING in the signed message
 * - Signature and public key are hex encoded (NOT base64)
 */
async function sendSltn(
  wallet: Wallet,
  to: string,
  amountSltn: number
): Promise<TransactionResult> {
  // Get current nonce
  const balanceInfo = await getBalance(wallet.address);
  const nonce = balanceInfo.nonce;
  
  // Convert to atomic units
  const amountAtomic = Math.floor(amountSltn * 1e9);
  const timestamp = Math.floor(Date.now() / 1000);
  
  // Create message for signing
  // CRITICAL: Use fast-json-stable-stringify for alphabetical key order
  // CRITICAL: Amount must be a STRING
  const txForSigning = {
    amount: String(amountAtomic),  // MUST be string
    from: wallet.address,
    memo: '',
    nonce,
    timestamp,
    to
  };
  
  // Sign the message
  const message = new TextEncoder().encode(stringify(txForSigning));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  // Build request body
  const request = {
    tx: {
      from: wallet.address,
      to,
      amount: amountAtomic,  // Can be number in request body
      timestamp,
      nonce,
      memo: ''
    },
    signature: bytesToHex(signature),      // Hex encoded (128 chars)
    public_key: bytesToHex(wallet.publicKey) // Hex encoded (64 chars)
  };
  
  // Submit transaction
  const res = await fetch(`${RPC_URL}/tx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request)
  });
  
  return res.json();
}

/**
 * Get transaction by hash
 */
async function getTransaction(hash: string): Promise<TransactionResult> {
  const res = await fetch(`${RPC_URL}/tx/${hash}`);
  return res.json();
}

/**
 * Wait for transaction confirmation
 */
async function waitForConfirmation(hash: string, timeoutMs = 30000): Promise<TransactionResult> {
  const start = Date.now();
  
  while (Date.now() - start < timeoutMs) {
    const tx = await getTransaction(hash);
    if (tx.status === 'confirmed') {
      return tx;
    }
    // Wait 2 seconds (one block time)
    await new Promise(r => setTimeout(r, 2000));
  }
  
  throw new Error('Transaction confirmation timeout');
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

async function main() {
  console.log('=== Sultan L1 SDK Example (TypeScript) ===\n');
  
  // Check network status
  console.log('📡 Checking network status...');
  const status = await getStatus();
  console.log(`   Block Height: ${status.block_height}`);
  console.log(`   Validators: ${status.validators}`);
  console.log(`   TPS Capacity: ${status.tps_capacity}`);
  console.log(`   Version: ${status.version}\n`);
  
  // Create a new wallet
  console.log('🔑 Creating new wallet...');
  const wallet = await createWallet();
  console.log(`   Address: ${wallet.address}`);
  console.log(`   Public Key: ${bytesToHex(wallet.publicKey).slice(0, 16)}...\n`);
  
  // Check balance
  console.log('💰 Checking balance...');
  const balance = await getBalanceSltn(wallet.address);
  console.log(`   Balance: ${balance} SLTN\n`);
  
  // Example: Send transaction (commented out - requires funded wallet)
  // console.log('📤 Sending 10 SLTN...');
  // const tx = await sendSltn(wallet, 'sultan1recipient...', 10);
  // console.log(`   TX Hash: ${tx.hash}`);
  // 
  // const confirmed = await waitForConfirmation(tx.hash);
  // console.log(`   Status: ${confirmed.status}`);
  
  console.log('✅ SDK example complete!');
  console.log('\nFor more examples, see: https://github.com/Sultan-Labs/DOCS/tree/main/examples');
}

// Run if executed directly
main().catch(console.error);

// ============================================================================
// EXPORTS (for use as a module)
// ============================================================================

export {
  createWallet,
  importWallet,
  getStatus,
  getBalance,
  getBalanceSltn,
  sendSltn,
  getTransaction,
  waitForConfirmation,
  bytesToHex,
  hexToBytes,
  Wallet,
  Balance,
  Status,
  TransactionResult
};
