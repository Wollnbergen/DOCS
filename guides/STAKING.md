# Staking Guide

Delegate SLTN tokens and earn staking rewards on Sultan L1.

## Overview

Sultan L1 uses a **Proof-of-Stake** consensus mechanism:

- **Staking APY:** ~13.33% (at 30% network staked)
- **Minimum stake:** 10,000 SLTN (to become a validator)
- **Delegation:** Any amount (to delegate to existing validators)
- **Unbonding period:** 21 days
- **Zero fees:** All staking operations are FREE

## APY Calculation

APY depends on the percentage of total supply staked:

```
APY = Inflation Rate / Stake Percentage
APY = 4% / 30% = 13.33%
```

| % Staked | APY |
|----------|-----|
| 20% | 20% |
| 30% | 13.33% |
| 40% | 10% |
| 50% | 8% |

## Query Validators

### List All Validators

```typescript
const validators = await fetch('https://rpc.sltn.io/staking/validators')
  .then(r => r.json());

// {
//   "validators": [
//     {
//       "address": "sultanvaloper1...",
//       "moniker": "Validator One",
//       "stake": 50000000000000,
//       "voting_power": 16.67,
//       "commission": 0.05,
//       "status": "active",
//       "uptime": 99.9
//     }
//   ],
//   "total_validators": 6,
//   "total_stake": 300000000000000
// }
```

### Get Staking Statistics

```typescript
const stats = await fetch('https://rpc.sltn.io/staking/statistics')
  .then(r => r.json());

// {
//   "total_staked": 150000000000000000,
//   "staking_ratio": 0.30,
//   "current_apy": 0.1333,
//   "total_validators": 6,
//   "active_validators": 6
// }
```

## Delegate to a Validator

```typescript
async function delegate(
  wallet,
  validatorAddress: string,
  amountSltn: number
) {
  const request = {
    delegator: wallet.address,
    validator: validatorAddress,
    amount: Math.floor(amountSltn * 1e9)
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/staking/delegate', {
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

// Delegate 1000 SLTN to a validator
const result = await delegate(
  wallet,
  'sultanvaloper1validator...',
  1000
);

console.log('Delegation TX:', result.hash);
```

## Query My Delegations

```typescript
const delegations = await fetch(
  `https://rpc.sltn.io/staking/delegations/${wallet.address}`
).then(r => r.json());

// {
//   "delegator": "sultan1...",
//   "delegations": [
//     {
//       "validator": "sultanvaloper1...",
//       "amount": 1000000000000,
//       "rewards_pending": 5000000000,
//       "shares": 1000000000000
//     }
//   ],
//   "total_delegated": 1000000000000,
//   "total_rewards_pending": 5000000000
// }
```

## Claim Rewards

```typescript
async function claimRewards(
  wallet,
  validatorAddress: string
) {
  const request = {
    delegator: wallet.address,
    validator: validatorAddress
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/staking/withdraw_rewards', {
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

// Claim rewards from a validator
const result = await claimRewards(wallet, 'sultanvaloper1...');
console.log('Rewards claimed:', result.amount / 1e9, 'SLTN');
```

## Undelegate (Unstake)

Undelegating starts a 21-day unbonding period:

```typescript
async function undelegate(
  wallet,
  validatorAddress: string,
  amountSltn: number
) {
  const request = {
    delegator: wallet.address,
    validator: validatorAddress,
    amount: Math.floor(amountSltn * 1e9)
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/staking/undelegate', {
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

// Unstake 500 SLTN
const result = await undelegate(wallet, 'sultanvaloper1...', 500);
console.log('Unbonding completes at:', new Date(result.completion_time * 1000));
```

## Set Reward Wallet

Route staking rewards to a different address:

```typescript
async function setRewardWallet(
  wallet,
  rewardAddress: string
) {
  const timestamp = Math.floor(Date.now() / 1000);
  
  const request = {
    delegator: wallet.address,
    reward_address: rewardAddress,
    timestamp
  };
  
  const message = new TextEncoder().encode(JSON.stringify(request));
  const signature = await ed25519.signAsync(message, wallet.privateKey);
  
  const res = await fetch('https://rpc.sltn.io/staking/set_reward_wallet', {
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

// Set rewards to go to a different address
await setRewardWallet(wallet, 'sultan1rewards...');
```

## Query Reward Wallet

```typescript
const rewardWallet = await fetch(
  `https://rpc.sltn.io/staking/reward_wallet/${wallet.address}`
).then(r => r.json());

// {
//   "delegator": "sultan1...",
//   "reward_address": "sultan1rewards...",
//   "set_at": 1735689600
// }
```

## Choosing a Validator

When selecting a validator to delegate to, consider:

| Factor | Description |
|--------|-------------|
| **Commission** | Lower commission = more rewards for you |
| **Uptime** | Higher uptime = more consistent rewards |
| **Stake** | Well-staked validators are more secure |
| **Voting Power** | Diversify to avoid concentration |

## Best Practices

1. **Diversify** - Delegate to multiple validators
2. **Monitor uptime** - Check validator performance regularly
3. **Claim rewards** - Compound by re-delegating rewards
4. **Plan unstaking** - Remember the 21-day unbonding period

## Become a Validator

To run your own validator (minimum 10,000 SLTN):

1. See the [Validator Guide](https://github.com/Sultan-Labs/0xv7/blob/main/VALIDATOR_GUIDE.md)
2. Download the sultan-node binary
3. Configure with your stake and validator address
4. Connect to the network

## Next Steps

- [API Reference](../api/API_REFERENCE.md) - Complete staking endpoints
- [SDK Documentation](SDK.md) - Full SDK with staking examples
