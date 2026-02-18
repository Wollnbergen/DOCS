# Sultan L1 Developer Documentation

**Build DApps, DEXs, Wallets, and More on Sultan L1 - The Zero-Fee Blockchain**

[![Production](https://img.shields.io/badge/Network-Production-brightgreen)](https://rpc.sltn.io/status)
[![Zero Fees](https://img.shields.io/badge/Fees-$0-blue)](https://sltn.io)
[![80K TPS](https://img.shields.io/badge/TPS-80,000+-purple)](https://sltn.io)

## 🚀 Quick Start

```rust
// Rust SDK
use sultan_sdk::SultanSDK;

let sdk = SultanSDK::new_mainnet().await?;
let balance = sdk.get_balance_sltn("sultan1...").await?;
```

```typescript
// TypeScript/JavaScript
const balance = await fetch('https://rpc.sltn.io/balance/sultan1...')
  .then(r => r.json());
console.log(`Balance: ${balance.balance / 1e9} SLTN`);
```

```python
# Python
import requests
balance = requests.get('https://rpc.sltn.io/balance/sultan1...').json()
print(f"Balance: {balance['balance'] / 1e9} SLTN")
```

## 📚 Documentation

| Resource | Description |
|----------|-------------|
| [API Reference](api/API_REFERENCE.md) | Complete HTTP API documentation |
| [RPC Specification](api/RPC_SPECIFICATION.md) | All 38 RPC endpoints |
| [SDK Examples](examples/) | Code examples in Rust, TypeScript, Python |
| [Developer Guides](guides/) | Step-by-step integration guides |

## 🌐 Network Information

| Environment | RPC URL | Chain ID |
|-------------|---------|----------|
| **Mainnet** | `https://rpc.sltn.io` | `sultan-1` |
| **Testnet** | `https://testnet.sltn.io` | `sultan-testnet-1` |

## ⚡ Why Build on Sultan?

| Feature | Sultan L1 | Others |
|---------|-----------|--------|
| **Transaction Fees** | $0 (Zero forever) | $0.01 - $50+ |
| **Throughput** | 80,000+ TPS | 10-10,000 TPS |
| **Block Time** | 2 seconds | 2-15 seconds |
| **Finality** | Instant (1 block) | 1-15 minutes |
| **DEX** | Native (built-in) | Smart contracts |
| **Token Factory** | Native (1 API call) | Smart contracts |

## 🔑 Key Features for Developers

### Zero-Fee Architecture
All transactions on Sultan are **FREE** - no gas fees, no compute units, no hidden costs. Validators are funded through controlled 4% inflation.

### Native DEX
Built-in AMM with:
- Constant product formula (x*y=k)
- 0.3% LP fee only (no gas!)
- Direct token swaps

### Token Factory
Create custom tokens in one API call:
```typescript
const token = await createToken({
  name: "My Token",
  symbol: "MTK",
  decimals: 6,
  initial_supply: 1000000
});
// Token denom: factory/sultan1.../MTK
```

### Cross-Chain Bridges
Native bridges to Bitcoin, Ethereum, Solana, and TON with zero Sultan-side fees.

## ⚠️ Critical: Signature Format

**All signed operations require:**

| Field | Format | Example |
|-------|--------|---------|
| `signature` | Hex encoded | 128 characters (64 bytes) |
| `public_key` | Hex encoded | 64 characters (32 bytes) |
| `amount` | String in signed message | `"1000000000"` |
| Key Order | Alphabetically sorted | Use `fast-json-stable-stringify` |

```javascript
// ✅ Correct message format (keys alphabetical, amount as string)
{"amount":"1000000000","from":"sultan1...","memo":"","nonce":0,"timestamp":1735689600,"to":"sultan1..."}
```

## 📦 Installation

### TypeScript/JavaScript
```bash
npm install @noble/ed25519 bech32 fast-json-stable-stringify
```

### Rust
```toml
[dependencies]
ed25519-dalek = "2.0"
bech32 = "0.11"
reqwest = { version = "0.12", features = ["json"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

### Python
```bash
pip install pynacl bech32 requests
```

## 📖 Guides

| Guide | Description |
|-------|-------------|
| [Quick Start](guides/QUICK_START.md) | Get started in 5 minutes |
| [Wallet Integration](guides/WALLET_INTEGRATION.md) | Connect dApps to Sultan Wallet |
| [Token Creation](guides/TOKEN_FACTORY.md) | Create custom tokens |
| [DEX Integration](guides/DEX_INTEGRATION.md) | Build on the native DEX |
| [Staking](guides/STAKING.md) | Delegate and earn rewards |

## 🔗 Links

- **Website:** [sltn.io](https://sltn.io)
- **RPC Status:** [rpc.sltn.io/status](https://rpc.sltn.io/status)
- **Block Explorer:** Coming Q1 2026
- **Discord:** [discord.gg/sultanchain](https://discord.gg/sultanchain)

## 📄 SDK Documentation

For comprehensive SDK documentation with full examples in all supported languages, see:
- [Complete SDK Documentation](guides/SDK.md) - 1300+ lines of examples

---

**Version:** 2.1  
**Updated:** February 2026  
**License:** MIT
