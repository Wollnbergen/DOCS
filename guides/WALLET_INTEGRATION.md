# Wallet Integration Guide

Connect your dApp to Sultan Wallet via WalletLink (deep links + encrypted WebSocket relay).

## Overview

Sultan Wallet supports two connection methods:

| Method | How It Works | Best For |
|--------|-------------|----------|
| **WalletLink** | Deep link → WebSocket relay → encrypted P2P | Cross-origin dApps, mobile |
| **Extension API** | `window.sultanWallet` injected provider | Same-origin, desktop |

WalletLink is the primary integration path. It works everywhere — desktop browsers, mobile browsers, and across different origins (e.g., your dApp on `yourdapp.com` connecting to Sultan Wallet at `wallet.sltn.io`).

## Quick Start

### 1. Copy the WalletLink Client

Copy [`walletLink.ts`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/api/walletLink.ts) from the reference app into your project. This is the complete client (~680 lines) with encryption, relay communication, and session management built in.

```bash
# From the reference app
curl -o src/api/walletLink.ts \
  https://raw.githubusercontent.com/Sultan-Labs/hodl-holdings/main/client/src/api/walletLink.ts
```

### 2. Connect a Wallet

```typescript
import { getWalletLink } from './api/walletLink';

const walletLink = getWalletLink();

// Generate session and get deep link URL
const { deepLinkUrl } = await walletLink.generateSession();

// Open wallet (redirect or new tab)
window.open(deepLinkUrl, '_blank');

// Wait for the user to approve in their wallet (up to 2 min timeout)
const address = await walletLink.waitForConnection();
console.log('Connected:', address);
// → "sultan1abc123..."
```

### 3. Request a Signature

```typescript
const { signature, publicKey } = await walletLink.signMessage(
  JSON.stringify({
    type: 'transfer',
    from: address,
    to: 'sultan1recipient...',
    amount: '1000000000', // 1 SLTN (9 decimals)
    timestamp: Date.now(),
    nonce: 0
  })
);
```

### 4. Submit Transaction

```typescript
const result = await fetch('https://rpc.sltn.io/transaction', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    tx: { from: address, to: 'sultan1recipient...', amount: '1000000000', timestamp, nonce: 0 },
    signature,
    public_key: publicKey
  })
});
```

## Full API Reference

### `getWalletLink(): WalletLinkClient`

Returns the singleton WalletLink client instance.

### `walletLink.generateSession(): Promise<{ deepLinkUrl: string; sessionId: string }>`

Creates a new encrypted session on the relay server and returns a deep link URL. The URL points to `https://wallet.sltn.io/connect?session=<encoded-data>` which the wallet uses to join the session.

### `walletLink.waitForConnection(timeoutMs?: number): Promise<string>`

Waits for the wallet to approve the connection. Returns the wallet address. Default timeout is 120 seconds.

### `walletLink.signMessage(message: string): Promise<{ signature: string; publicKey: string }>`

Sends a signature request to the connected wallet. The wallet will show an approval prompt to the user. Returns the Ed25519 signature and public key (both hex-encoded).

### `walletLink.isConnected(): boolean`

Returns whether a wallet is currently connected.

### `walletLink.getAddress(): string | null`

Returns the connected wallet address, or null.

### `walletLink.getPublicKey(): string | null`

Returns the connected wallet's public key (hex), or null.

### `walletLink.disconnect(): Promise<void>`

Ends the session and closes the relay connection.

### `walletLink.on(handler): () => void`

Subscribe to events. Returns an unsubscribe function.

```typescript
const unsubscribe = walletLink.on((event) => {
  switch (event.type) {
    case 'connected':
      console.log('Wallet connected:', event.data);
      break;
    case 'disconnected':
      console.log('Wallet disconnected');
      break;
    case 'error':
      console.error('Error:', event.data);
      break;
  }
});

// Later: unsubscribe()
```

### `walletLink.restoreSession(): Promise<boolean>`

Attempts to restore a previously saved session from `sessionStorage`. Returns true if a valid session was restored (sessions expire after 10 minutes).

## React Integration

See the reference app for a complete React implementation:

- [`useWallet.ts`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/lib/wallet.ts) — React hook with `connectPWA()`, `disconnect()`, state management
- [`WalletConnectModal.tsx`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/components/WalletConnectModal.tsx) — Connection UI with auto-open wallet tab

### Minimal React Example

```tsx
import { useState } from 'react';
import { getWalletLink } from './api/walletLink';

function ConnectButton() {
  const [address, setAddress] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);

  const connect = async () => {
    setConnecting(true);
    try {
      const walletLink = getWalletLink();
      const { deepLinkUrl } = await walletLink.generateSession();

      // Open wallet in new tab (must happen during user gesture)
      window.open(deepLinkUrl, '_blank');

      // Wait for approval
      const addr = await walletLink.waitForConnection();
      setAddress(addr);
    } catch (err) {
      console.error('Connection failed:', err);
    } finally {
      setConnecting(false);
    }
  };

  if (address) {
    return <span>{address.slice(0, 12)}...{address.slice(-6)}</span>;
  }

  return (
    <button onClick={connect} disabled={connecting}>
      {connecting ? 'Waiting for wallet...' : 'Connect Wallet'}
    </button>
  );
}
```

## Extension API (Optional)

If the Sultan Wallet browser extension is installed, you can also connect via the injected provider:

```typescript
if (window.sultanWallet) {
  const account = await window.sultanWallet.connect();
  console.log('Address:', account.address);

  const signed = await window.sultanWallet.signTransaction({
    to: 'sultan1recipient...',
    amount: '1000000000'
  });
}
```

The reference app auto-detects the extension and falls back to WalletLink when it's not available.

## Protocol Details

### How WalletLink Works

```
┌──────────┐         ┌──────────────┐        ┌────────────┐
│  Your    │   WSS   │  Relay       │  WSS   │  Sultan    │
│  dApp    │◄───────►│  Server      │◄──────►│  Wallet    │
│          │         │  (Fly.io)    │        │  (PWA)     │
└──────────┘         └──────────────┘        └────────────┘
     │                                              │
     │  1. session_init ──────────────────►          │
     │  2. ◄── session_ack (machineId)               │
     │  3. Generate deep link with session data      │
     │  4. Open wallet.sltn.io/connect?session=...  │
     │                                    5. Parse session ──►│
     │                              6. session_join ──────►   │
     │  7. ◄── connect_response (address, pubkey)    │
     │  8. sign_message_request ──────────────────►  │
     │  9. ◄── sign_message_response (signature)     │
```

All messages between dApp and wallet are **AES-256-GCM encrypted** using a key derived via HKDF-SHA256 from the shared session key. The relay server cannot read message contents.

### Relay Server

- **URL:** `wss://sultan-walletlink-relay.fly.dev`
- **Health:** `https://sultan-walletlink-relay.fly.dev/health`
- **Source:** [`Sultan-Labs/PWA/server/relay-server.ts`](https://github.com/Sultan-Labs/PWA/tree/main/server)

### Deep Link Format

```
https://wallet.sltn.io/connect?session=<encoded-sultan-url>
```

Where `<encoded-sultan-url>` is URL-encoded:

```
sultan://wl?s=<sessionId>&k=<base64Key>&b=<relayUrl>&m=<machineId>&n=<appName>&o=<origin>
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `s` | Yes | Session ID (UUID v4) |
| `k` | Yes | Session key (32 bytes, URL-safe base64) |
| `b` | Yes | Relay WebSocket URL |
| `m` | No | Relay machine ID (for multi-instance routing) |
| `n` | No | Your app name (shown in wallet approval screen) |
| `o` | No | Your app origin (shown in wallet approval screen) |

### Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| `session_init` | dApp → Relay | Initialize session |
| `session_ack` | Relay → dApp | Session confirmed (includes `machineId`) |
| `session_join` | Wallet → Relay | Wallet joins session |
| `connect_response` | Wallet → dApp | Wallet address + public key |
| `sign_message_request` | dApp → Wallet | Request signature |
| `sign_message_response` | Wallet → dApp | Ed25519 signature |
| `session_end` | Either → Relay | End session |
| `heartbeat` | Either → Relay | Keep connection alive |

### Encryption

- **Algorithm:** AES-256-GCM
- **Key Derivation:** HKDF-SHA256 from 32-byte session key
- **IV:** Random 12 bytes per message
- **Format:** Base64-encoded `iv + ciphertext`
- **Wrapped in:** JSON envelope `{ sessionId, type, data: "<encrypted>" }` for relay routing

## Sultan L1 Basics

| Property | Value |
|----------|-------|
| Chain ID | `sultan-1` |
| Token | SLTN |
| Decimals | 9 |
| Address prefix | `sultan1` |
| Signature | Ed25519 |
| Gas fees | $0.00 (zero-fee) |
| RPC | `https://rpc.sltn.io` |

## Reference Implementation

**HODL Holdings** is the fully working reference dApp:

- **Live:** [hodlholdings.com](https://hodlholdings.com)
- **Source:** [Sultan-Labs/hodl-holdings](https://github.com/Sultan-Labs/hodl-holdings)

Key files:
| File | Purpose |
|------|---------|
| [`client/src/api/walletLink.ts`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/api/walletLink.ts) | Complete WalletLink client |
| [`client/src/lib/wallet.ts`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/lib/wallet.ts) | React `useWallet()` hook |
| [`client/src/components/WalletConnectModal.tsx`](https://github.com/Sultan-Labs/hodl-holdings/blob/main/client/src/components/WalletConnectModal.tsx) | Connection modal UI |

## Security

1. **End-to-end encrypted** — Relay never sees message contents
2. **Session keys are ephemeral** — Generated per connection, not stored long-term
3. **User approval required** — Wallet shows approval screen for connections and signatures
4. **HTTPS only** — Always use HTTPS in production
5. **Validate addresses** — Always validate `sultan1...` addresses before use
6. **Never ask for private keys** — WalletLink handles signing securely in the wallet

## Next Steps

- [Quick Start Guide](QUICK_START.md)
- [API Reference](../api/API_REFERENCE.md)
- [RPC Specification](../api/RPC_SPECIFICATION.md)
- [Token Factory Guide](TOKEN_FACTORY.md)
