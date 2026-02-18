# Quick Start Guide

Get started building on Sultan L1 in 5 minutes.

## Prerequisites

- Basic understanding of blockchain concepts
- Development environment (Node.js 18+, Python 3.9+, or Rust 1.75+)

## Network Information

| Network | RPC URL | Chain ID |
|---------|---------|----------|
| **Mainnet** | `https://rpc.sltn.io` | `sultan-1` |
| **Testnet** | `https://testnet.sltn.io` | `sultan-testnet-1` |

## Step 1: Check Network Status

Verify the network is operational:

```bash
curl https://rpc.sltn.io/status
```

Expected response:
```json
{
  "node_id": "sultan-validator-1",
  "block_height": 125000,
  "validators": 6,
  "shard_count": 20,
  "tps_capacity": 80000
}
```

## Step 2: Query a Balance

```bash
curl https://rpc.sltn.io/balance/sultan15g5e8xyz...
```

Response:
```json
{
  "address": "sultan15g5e8xyz...",
  "balance": 500000000000000000,
  "nonce": 5
}
```

> **Note:** Balance is in atomic units. Divide by `10^9` to get SLTN amount.

## Step 3: Set Up Your Development Environment

### TypeScript/JavaScript

```bash
mkdir my-sultan-app && cd my-sultan-app
npm init -y
npm install @noble/ed25519 bech32 fast-json-stable-stringify
```

### Python

```bash
pip install pynacl bech32 requests
```

### Rust

```toml
# Cargo.toml
[dependencies]
ed25519-dalek = "2.0"
bech32 = "0.11"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Step 4: Create a Wallet

### TypeScript

```typescript
import * as ed25519 from '@noble/ed25519';
import { bech32 } from 'bech32';

async function createWallet() {
  const privateKey = ed25519.utils.randomPrivateKey();
  const publicKey = await ed25519.getPublicKeyAsync(privateKey);
  
  // Derive address
  const hash = await crypto.subtle.digest('SHA-256', publicKey);
  const addressBytes = new Uint8Array(hash).slice(0, 20);
  const words = bech32.toWords(addressBytes);
  const address = bech32.encode('sultan', words);
  
  return { privateKey, publicKey, address };
}

const wallet = await createWallet();
console.log('Address:', wallet.address);
```

### Python

```python
from nacl.signing import SigningKey
import hashlib
from bech32 import bech32_encode, convertbits

signing_key = SigningKey.generate()
public_key = signing_key.verify_key.encode()

# Derive address
hash_bytes = hashlib.sha256(public_key).digest()[:20]
data = convertbits(list(hash_bytes), 8, 5)
address = bech32_encode("sultan", data)

print(f"Address: {address}")
```

## Step 5: Send a Transaction

> ⚠️ **Critical:** See the [Signature Format](#signature-format) section before sending transactions.

### TypeScript

```typescript
import stringify from 'fast-json-stable-stringify';

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

async function sendSltn(wallet, to, amountSltn) {
  // Get nonce
  const balance = await fetch(`https://rpc.sltn.io/balance/${wallet.address}`)
    .then(r => r.json());
  
  // Create message for signing
  const txForSigning = {
    amount: String(Math.floor(amountSltn * 1e9)),  // STRING!
    from: wallet.address,
    memo: '',
    nonce: balance.nonce,
    timestamp: Math.floor(Date.now() / 1000),
    to
  };
  
  // Sign (CRITICAL: use stringify for alphabetical keys)
  const message = new TextEncoder().encode(stringify(txForSigning));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  // Submit
  const res = await fetch('https://rpc.sltn.io/tx', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      tx: {
        from: wallet.address,
        to,
        amount: parseInt(txForSigning.amount),
        timestamp: txForSigning.timestamp,
        nonce: balance.nonce,
        memo: ''
      },
      signature: bytesToHex(signature),
      public_key: bytesToHex(wallet.publicKey)
    })
  });
  
  return res.json();
}
```

## Signature Format

**All signed operations require proper signature format:**

| Requirement | Details |
|-------------|---------|
| **Encoding** | Signatures and public keys are **hex encoded** (NOT base64) |
| **Key Order** | JSON message keys must be **alphabetically sorted** |
| **Amount** | Amount must be a **string** in the signed message |
| **Signature Length** | 128 hex characters (64 bytes) |
| **Public Key Length** | 64 hex characters (32 bytes) |

**Example signed message (exact format):**
```json
{"amount":"1000000000","from":"sultan1...","memo":"","nonce":0,"timestamp":1735689600,"to":"sultan1..."}
```

## Next Steps

- [Complete SDK Documentation](SDK.md) - Full API with all features
- [API Reference](../api/API_REFERENCE.md) - All HTTP endpoints
- [Token Factory Guide](TOKEN_FACTORY.md) - Create custom tokens
- [DEX Integration](DEX_INTEGRATION.md) - Build on the native DEX
- [Code Examples](../examples/) - Working examples in all languages

## Support

- **Discord:** [discord.gg/sultanchain](https://discord.gg/sultanchain)
- **GitHub Issues:** [Sultan-Labs/DOCS](https://github.com/Sultan-Labs/DOCS/issues)
