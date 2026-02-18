# Token Factory Guide

Create custom tokens on Sultan L1 in one API call - no smart contracts needed!

## Overview

Sultan L1 has a **native Token Factory** module that allows anyone to create, mint, and manage custom tokens. Key benefits:

- **Zero fees** - Creating and transferring tokens is FREE
- **No smart contracts** - Direct protocol-level support
- **Instant deployment** - Token is live in one transaction

## Token Denom Format

All custom tokens follow this format:
```
factory/{creator_address}/{symbol}
```

Example: `factory/sultan1abc123.../MTK`

## Create a Token

### TypeScript

```typescript
import * as ed25519 from '@noble/ed25519';
import stringify from 'fast-json-stable-stringify';

async function createToken(
  wallet,
  name: string,
  symbol: string,
  decimals: number,
  totalSupply: number
) {
  const request = {
    creator: wallet.address,
    name,
    symbol,
    decimals,
    total_supply: Math.floor(totalSupply * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/tokens/create', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: bytesToHex(signature),
      public_key: bytesToHex(wallet.publicKey)
    })
  });
  
  return res.json();
}

// Create "My Token" with 1 million supply
const token = await createToken(
  wallet,
  'My Token',
  'MTK',
  6,        // 6 decimals
  1000000   // 1 million tokens
);

console.log('Token denom:', token.denom);
// factory/sultan1abc.../MTK
```

### Python

```python
import json
import requests

def create_token(wallet, name, symbol, decimals, total_supply):
    request = {
        "creator": wallet.address,
        "name": name,
        "symbol": symbol,
        "decimals": decimals,
        "total_supply": int(total_supply * (10 ** decimals))
    }
    
    signature, pubkey = wallet.sign(request)
    
    res = requests.post(
        "https://rpc.sltn.io/tokens/create",
        json={
            **request,
            "signature": signature,
            "public_key": pubkey
        }
    )
    return res.json()

# Create token
token = create_token(wallet, "My Token", "MTK", 6, 1000000)
print(f"Token denom: {token['denom']}")
```

### cURL

```bash
curl -X POST https://rpc.sltn.io/tokens/create \
  -H "Content-Type: application/json" \
  -d '{
    "creator": "sultan1abc...",
    "name": "My Token",
    "symbol": "MTK",
    "decimals": 6,
    "total_supply": 1000000000000,
    "signature": "abc123...",
    "public_key": "def456..."
  }'
```

## Transfer Tokens

```typescript
async function transferToken(
  wallet,
  denom: string,
  to: string,
  amount: number,
  decimals: number
) {
  const request = {
    denom,
    from: wallet.address,
    to,
    amount: Math.floor(amount * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/tokens/transfer', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: bytesToHex(signature),
      public_key: bytesToHex(wallet.publicKey)
    })
  });
  
  return res.json();
}

// Transfer 100 MTK tokens
await transferToken(
  wallet,
  'factory/sultan1abc.../MTK',
  'sultan1recipient...',
  100,
  6
);
```

## Query Token Balance

```typescript
// GET /tokens/{denom}/balance/{address}
// Note: denom must be URL-encoded
const denom = encodeURIComponent('factory/sultan1abc.../MTK');
const balance = await fetch(
  `https://rpc.sltn.io/tokens/${denom}/balance/sultan1xyz...`
).then(r => r.json());

console.log('Balance:', balance.balance);
```

## Query Token Metadata

```typescript
const denom = encodeURIComponent('factory/sultan1abc.../MTK');
const metadata = await fetch(
  `https://rpc.sltn.io/tokens/${denom}/metadata`
).then(r => r.json());

// {
//   "denom": "factory/sultan1abc.../MTK",
//   "name": "My Token",
//   "symbol": "MTK",
//   "decimals": 6,
//   "total_supply": 1000000000000,
//   "creator": "sultan1abc..."
// }
```

## List All Tokens

```typescript
const tokens = await fetch('https://rpc.sltn.io/tokens/list').then(r => r.json());

// {
//   "tokens": [
//     { "denom": "factory/sultan1.../MTK", "name": "My Token", "symbol": "MTK" },
//     { "denom": "factory/sultan1.../ABC", "name": "ABC Token", "symbol": "ABC" }
//   ],
//   "total": 156
// }
```

## Mint Additional Tokens

Only the creator can mint additional supply:

```typescript
async function mintTokens(wallet, denom, amount, decimals) {
  const request = {
    denom,
    minter: wallet.address,
    amount: Math.floor(amount * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  return fetch('https://rpc.sltn.io/tokens/mint', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: bytesToHex(signature),
      public_key: bytesToHex(wallet.publicKey)
    })
  }).then(r => r.json());
}
```

## Burn Tokens

```typescript
async function burnTokens(wallet, denom, amount, decimals) {
  const request = {
    denom,
    burner: wallet.address,
    amount: Math.floor(amount * Math.pow(10, decimals))
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  return fetch('https://rpc.sltn.io/tokens/burn', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...request,
      signature: bytesToHex(signature),
      public_key: bytesToHex(wallet.publicKey)
    })
  }).then(r => r.json());
}
```

## Best Practices

1. **Choose unique symbols** - Symbols should be 3-6 uppercase letters
2. **Set appropriate decimals** - 6-9 decimals is common (6 for tokens, 9 for SLTN-like)
3. **Consider total supply** - Remember to account for decimals in total supply
4. **Save the denom** - You'll need the full denom path for all operations

## Next Steps

- [DEX Integration](DEX_INTEGRATION.md) - List your token on the native DEX
- [API Reference](../api/API_REFERENCE.md) - Complete API documentation
