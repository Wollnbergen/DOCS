//! Sultan L1 SDK Example - Rust
//!
//! This example demonstrates how to interact with Sultan L1 blockchain using Rust.
//! 
//! ## Setup
//! 
//! Add these dependencies to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! ed25519-dalek = "2.0"
//! bech32 = "0.11"
//! reqwest = { version = "0.12", features = ["json"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! sha2 = "0.10"
//! hex = "0.4"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ## Usage
//! 
//! ```bash
//! cargo run --example sdk_example
//! ```

use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use bech32::{Bech32, Hrp};

const RPC_URL: &str = "https://rpc.sltn.io";

// ============================================================================
// WALLET
// ============================================================================

#[derive(Debug)]
pub struct Wallet {
    signing_key: SigningKey,
    pub public_key: VerifyingKey,
    pub address: String,
}

impl Wallet {
    /// Create a new random wallet
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let public_key = signing_key.verifying_key();
        
        // Derive address: SHA256(pubkey)[0:20] -> bech32("sultan")
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        let addr_bytes = &hash[..20];
        
        let hrp = Hrp::parse("sultan").expect("valid hrp");
        let address = bech32::encode::<Bech32>(hrp, addr_bytes).expect("bech32 encode");
        
        Self { signing_key, public_key, address }
    }
    
    /// Import wallet from private key hex
    pub fn from_private_key(hex_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let key_bytes = hex::decode(hex_key)?;
        let signing_key = SigningKey::try_from(key_bytes.as_slice())?;
        let public_key = signing_key.verifying_key();
        
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        let addr_bytes = &hash[..20];
        
        let hrp = Hrp::parse("sultan").expect("valid hrp");
        let address = bech32::encode::<Bech32>(hrp, addr_bytes).expect("bech32 encode");
        
        Ok(Self { signing_key, public_key, address })
    }
    
    /// Sign a message (returns hex-encoded signature)
    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        hex::encode(signature.to_bytes())
    }
    
    /// Get public key as hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.as_bytes())
    }
}

// ============================================================================
// API TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u128,
    pub nonce: u64,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub node_id: String,
    pub block_height: u64,
    pub validators: u32,
    pub uptime_seconds: u64,
    pub version: String,
    pub shard_count: u32,
    pub tps_capacity: u32,
}

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u128,
    pub block_height: Option<u64>,
    pub status: String,
}

#[derive(Debug, Serialize)]
struct TransactionForSigning {
    amount: String,  // MUST be string for signing
    from: String,
    memo: String,
    nonce: u64,
    timestamp: u64,
    to: String,
}

#[derive(Debug, Serialize)]
struct TransactionRequest {
    tx: TransactionBody,
    signature: String,
    public_key: String,
}

#[derive(Debug, Serialize)]
struct TransactionBody {
    from: String,
    to: String,
    amount: u128,
    timestamp: u64,
    nonce: u64,
    memo: String,
}

// ============================================================================
// SDK CLIENT
// ============================================================================

pub struct SultanSDK {
    client: reqwest::Client,
    base_url: String,
}

impl SultanSDK {
    /// Create SDK instance for mainnet
    pub fn new_mainnet() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: RPC_URL.to_string(),
        }
    }
    
    /// Create SDK instance for testnet
    pub fn new_testnet() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://testnet.sltn.io".to_string(),
        }
    }
    
    /// Create SDK instance with custom RPC URL
    pub fn new(rpc_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: rpc_url.to_string(),
        }
    }
    
    /// Get network status
    pub async fn get_status(&self) -> Result<StatusResponse, reqwest::Error> {
        let url = format!("{}/status", self.base_url);
        self.client.get(&url).send().await?.json().await
    }
    
    /// Get balance for an address (in atomic units)
    pub async fn get_balance(&self, address: &str) -> Result<BalanceResponse, reqwest::Error> {
        let url = format!("{}/balance/{}", self.base_url, address);
        self.client.get(&url).send().await?.json().await
    }
    
    /// Get balance in SLTN (human-readable)
    pub async fn get_balance_sltn(&self, address: &str) -> Result<f64, reqwest::Error> {
        let balance = self.get_balance(address).await?;
        Ok(balance.balance as f64 / 1_000_000_000.0)
    }
    
    /// Send SLTN tokens
    pub async fn send_sltn(
        &self,
        wallet: &Wallet,
        to: &str,
        amount_sltn: f64,
    ) -> Result<TransactionResponse, Box<dyn std::error::Error>> {
        // Get current nonce
        let balance = self.get_balance(&wallet.address).await?;
        let nonce = balance.nonce;
        
        // Convert to atomic units
        let amount_atomic = (amount_sltn * 1_000_000_000.0) as u128;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Create message for signing (CRITICAL: alphabetical keys, amount as string)
        let tx_for_signing = TransactionForSigning {
            amount: amount_atomic.to_string(),
            from: wallet.address.clone(),
            memo: String::new(),
            nonce,
            timestamp,
            to: to.to_string(),
        };
        
        // Sign with deterministic JSON (serde_json sorts keys alphabetically by default)
        let message = serde_json::to_string(&tx_for_signing)?;
        let signature = wallet.sign(message.as_bytes());
        
        // Build request
        let request = TransactionRequest {
            tx: TransactionBody {
                from: wallet.address.clone(),
                to: to.to_string(),
                amount: amount_atomic,
                timestamp,
                nonce,
                memo: String::new(),
            },
            signature,
            public_key: wallet.public_key_hex(),
        };
        
        // Send transaction
        let url = format!("{}/tx", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionResponse, reqwest::Error> {
        let url = format!("{}/tx/{}", self.base_url, hash);
        self.client.get(&url).send().await?.json().await
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sultan L1 SDK Example ===\n");
    
    // Initialize SDK
    let sdk = SultanSDK::new_mainnet();
    
    // Check network status
    println!("📡 Checking network status...");
    let status = sdk.get_status().await?;
    println!("   Block Height: {}", status.block_height);
    println!("   Validators: {}", status.validators);
    println!("   TPS Capacity: {}", status.tps_capacity);
    println!("   Version: {}\n", status.version);
    
    // Create a new wallet
    println!("🔑 Creating new wallet...");
    let wallet = Wallet::new();
    println!("   Address: {}", wallet.address);
    println!("   Public Key: {}...\n", &wallet.public_key_hex()[..16]);
    
    // Check balance
    println!("💰 Checking balance...");
    let balance = sdk.get_balance_sltn(&wallet.address).await?;
    println!("   Balance: {} SLTN\n", balance);
    
    // Example: Send transaction (commented out - requires funded wallet)
    // println!("📤 Sending 10 SLTN...");
    // let tx = sdk.send_sltn(&wallet, "sultan1recipient...", 10.0).await?;
    // println!("   TX Hash: {}", tx.hash);
    
    println!("✅ SDK example complete!");
    println!("\nFor more examples, see: https://github.com/Sultan-Labs/DOCS/tree/main/examples");
    
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(wallet.address.starts_with("sultan1"));
        assert_eq!(wallet.public_key_hex().len(), 64);
    }
    
    #[test]
    fn test_wallet_signing() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign(message);
        assert_eq!(signature.len(), 128); // 64 bytes = 128 hex chars
    }
    
    #[tokio::test]
    async fn test_sdk_status() {
        let sdk = SultanSDK::new_mainnet();
        let status = sdk.get_status().await;
        assert!(status.is_ok());
    }
}
