# DEX Integration Guide

Build on Sultan L1's native AMM DEX with zero gas fees.

## Overview

Sultan L1 has a **native DEX** built directly into the protocol:

- **Zero gas fees** - Only 0.3% LP fee on swaps
- **Constant product AMM** - Standard x*y=k formula
- **Instant swaps** - 2-second finality
- **Direct integration** - No smart contracts needed

## Key Concepts

### Pair IDs

Pair IDs follow the format: `{token_a}-{token_b}` (alphabetically ordered)

Examples:
- `sltn-MTK` (SLTN/MTK pair)
- `sltn-factory/sultan1.../ABC` (SLTN/ABC pair)

### Fees

| Fee Type | Amount | Recipient |
|----------|--------|-----------|
| Gas Fee | **$0** | N/A |
| LP Fee | 0.3% | Liquidity providers |

## Query Pool Information

### Get Pool Details

```typescript
const pool = await fetch('https://rpc.sltn.io/dex/pool/sltn-MTK')
  .then(r => r.json());

// {
//   "pair_id": "sltn-MTK",
//   "token_a": "sltn",
//   "token_b": "factory/sultan1.../MTK",
//   "reserve_a": 1500000000000000,
//   "reserve_b": 750000000000,
//   "total_lp_tokens": 1000000000000,
//   "fee_bps": 30,
//   "volume_24h": 50000000000000
// }
```

### Get Current Price

```typescript
const price = await fetch('https://rpc.sltn.io/dex/price/sltn-MTK')
  .then(r => r.json());

// {
//   "pair_id": "sltn-MTK",
//   "price_a_to_b": 0.5,    // 1 SLTN = 0.5 MTK
//   "price_b_to_a": 2.0,    // 1 MTK = 2 SLTN
//   "reserve_a": 1500000000000000,
//   "reserve_b": 750000000000
// }
```

### List All Pools

```typescript
const pools = await fetch('https://rpc.sltn.io/dex/pools').then(r => r.json());

// {
//   "pools": [
//     { "pair_id": "sltn-MTK", "reserve_a": 1500000000000000, "volume_24h": 50000000000000 }
//   ],
//   "total": 25
// }
```

## Swap Tokens

### Calculate Expected Output

Before swapping, calculate the expected output using the constant product formula:

```typescript
function calculateSwapOutput(
  inputAmount: number,
  reserveIn: number,
  reserveOut: number,
  feeBps: number = 30  // 0.3%
): number {
  const inputWithFee = inputAmount * (10000 - feeBps);
  const numerator = inputWithFee * reserveOut;
  const denominator = (reserveIn * 10000) + inputWithFee;
  return Math.floor(numerator / denominator);
}

// Example: Swap 100 SLTN for MTK
const outputAmount = calculateSwapOutput(
  100_000_000_000,  // 100 SLTN
  1500000000000000, // Reserve SLTN
  750000000000      // Reserve MTK
);
console.log('Expected output:', outputAmount / 1e6, 'MTK');
```

### Execute Swap

```typescript
async function swap(
  wallet,
  inputDenom: string,
  outputDenom: string,
  inputAmount: number,
  minOutput: number  // Slippage protection
) {
  const request = {
    user: wallet.address,
    input_denom: inputDenom,
    output_denom: outputDenom,
    input_amount: inputAmount,
    min_output: minOutput
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/dex/swap', {
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

// Swap 100 SLTN for MTK with 5% slippage tolerance
const price = await fetch('https://rpc.sltn.io/dex/price/sltn-MTK').then(r => r.json());
const expectedOutput = 100 * price.price_a_to_b;
const minOutput = Math.floor(expectedOutput * 0.95 * 1e6); // 5% slippage

const result = await swap(
  wallet,
  'sltn',
  'factory/sultan1.../MTK',
  100_000_000_000,  // 100 SLTN
  minOutput
);

console.log('Received:', result.output_amount / 1e6, 'MTK');
```

## Provide Liquidity

### Add Liquidity

```typescript
async function addLiquidity(
  wallet,
  pairId: string,
  amountA: number,
  amountB: number,
  minLpTokens: number
) {
  const request = {
    user: wallet.address,
    pair_id: pairId,
    amount_a: amountA,
    amount_b: amountB,
    min_lp_tokens: minLpTokens
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/dex/add_liquidity', {
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

// Add 1000 SLTN + 500 MTK liquidity
const result = await addLiquidity(
  wallet,
  'sltn-MTK',
  1000_000_000_000,  // 1000 SLTN
  500_000_000,       // 500 MTK
  0                  // Minimum LP tokens (0 for no slippage protection)
);

console.log('LP tokens received:', result.lp_tokens);
```

### Remove Liquidity

```typescript
async function removeLiquidity(
  wallet,
  pairId: string,
  lpTokenAmount: number,
  minAmountA: number,
  minAmountB: number
) {
  const request = {
    user: wallet.address,
    pair_id: pairId,
    lp_token_amount: lpTokenAmount,
    min_amount_a: minAmountA,
    min_amount_b: minAmountB
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/dex/remove_liquidity', {
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
```

## Create a New Trading Pair

Create a new liquidity pool for a token pair:

```typescript
async function createPair(
  wallet,
  tokenA: string,
  tokenB: string,
  initialAmountA: number,
  initialAmountB: number
) {
  const request = {
    creator: wallet.address,
    token_a: tokenA,
    token_b: tokenB,
    initial_amount_a: initialAmountA,
    initial_amount_b: initialAmountB
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/dex/create_pair', {
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

// Create SLTN/MTK pair with initial liquidity
const pair = await createPair(
  wallet,
  'sltn',
  'factory/sultan1.../MTK',
  1000_000_000_000,  // 1000 SLTN
  2000_000_000       // 2000 MTK (sets initial price)
);

console.log('Pair created:', pair.pair_id);
```

## Price Impact Calculation

Calculate price impact before executing large swaps:

```typescript
function calculatePriceImpact(
  inputAmount: number,
  reserveIn: number,
  reserveOut: number
): number {
  // Spot price before swap
  const spotPriceBefore = reserveOut / reserveIn;
  
  // Output after swap
  const output = calculateSwapOutput(inputAmount, reserveIn, reserveOut);
  
  // Effective price
  const effectivePrice = output / inputAmount;
  
  // Price impact
  const priceImpact = (spotPriceBefore - effectivePrice) / spotPriceBefore * 100;
  
  return priceImpact;
}

// Example: Calculate impact of 10,000 SLTN swap
const impact = calculatePriceImpact(
  10000_000_000_000,  // 10,000 SLTN
  1500000000000000,   // Reserve SLTN
  750000000000        // Reserve MTK
);

console.log('Price impact:', impact.toFixed(2), '%');
```

## Best Practices

1. **Always use slippage protection** - Set `min_output` to protect against front-running
2. **Check price impact** - Large swaps can have significant price impact
3. **Monitor reserves** - Check pool reserves before large trades
4. **Consider splitting large orders** - Break up large swaps to minimize impact

## Next Steps

- [Token Factory Guide](TOKEN_FACTORY.md) - Create tokens to list on DEX
- [API Reference](../api/API_REFERENCE.md) - Complete API documentation
