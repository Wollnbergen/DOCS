# Wallet Integration Guide

Connect your dApp to Sultan Wallet (browser extension and PWA).

## Overview

Sultan Wallet supports multiple connection methods:

| Platform | Method | Best For |
|----------|--------|----------|
| Desktop Browser | Extension | Desktop dApps |
| Mobile Browser | WalletLink/QR | Mobile dApps |
| Mobile App | Deep Link | Mobile-to-mobile |

## Quick Start

### Install SDK

```bash
npm install @sultan/wallet-sdk
```

### Basic Connection

```typescript
import { SultanWalletSDK } from '@sultan/wallet-sdk';

const wallet = new SultanWalletSDK();

// Connect - auto-detects best method
const account = await wallet.connect();
console.log('Connected:', account.address);

// Sign a transaction
const signed = await wallet.signTransaction({
  to: 'sultan1recipient...',
  amount: '1000000000', // 1 SLTN
  memo: 'Payment'
});

// Submit transaction
const result = await fetch('https://rpc.sltn.io/tx', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(signed)
}).then(r => r.json());

console.log('TX Hash:', result.hash);
```

## Connection Methods

### Auto-Detection (Recommended)

```typescript
const wallet = new SultanWalletSDK();
const account = await wallet.connect();

// SDK automatically:
// 1. Checks for browser extension
// 2. Falls back to WalletLink QR code
// 3. On mobile, uses deep link to Sultan Wallet app
```

### Force Specific Method

```typescript
// Force WalletLink even if extension is installed
const wallet = new SultanWalletSDK({
  forceWalletLink: true
});

// Check what's available
if (SultanWalletSDK.isExtensionAvailable()) {
  console.log('Extension detected');
}

if (SultanWalletSDK.isMobile()) {
  console.log('Mobile browser - will use deep link');
}
```

## Configuration Options

```typescript
const wallet = new SultanWalletSDK({
  // Force WalletLink even if extension is available
  forceWalletLink: false,
  
  // Custom relay server for WalletLink
  relayUrl: 'wss://relay.sltn.io',
  
  // Container ID for QR code or connect button
  qrContainerId: 'wallet-connect-container',
  
  // Auto-redirect to wallet app on mobile
  autoRedirectMobile: true,
  
  // Callbacks
  onQRReady: (qrData) => {
    console.log('QR code ready');
  },
  onDeepLinkReady: (url) => {
    console.log('Deep link:', url);
  },
  onWaiting: () => {
    console.log('Waiting for wallet approval...');
  }
});
```

## Event Handling

```typescript
// Account changed (user switched accounts in wallet)
wallet.on('accountChange', (account) => {
  console.log('New account:', account.address);
  // Update your app state
});

// Wallet disconnected
wallet.on('disconnect', () => {
  console.log('Wallet disconnected');
  // Clear session, show connect button
});

// Network changed
wallet.on('networkChange', (network) => {
  console.log('Network:', network.chainId);
});

// Clean disconnect
wallet.disconnect();
```

## Signing Transactions

### Simple Transfer

```typescript
const signed = await wallet.signTransaction({
  to: 'sultan1recipient...',
  amount: '1000000000',  // 1 SLTN in atomic units
  memo: 'Payment for services'
});
```

### With Full Control

```typescript
const signed = await wallet.signTransaction({
  to: 'sultan1recipient...',
  amount: '1000000000',
  memo: '',
  nonce: 5,  // Optional: specify nonce
  timestamp: Math.floor(Date.now() / 1000)  // Optional: specify timestamp
});
```

### Token Transfer

```typescript
const signed = await wallet.signTokenTransfer({
  denom: 'factory/sultan1.../MTK',
  to: 'sultan1recipient...',
  amount: '1000000'
});
```

## Signing Messages

For signature verification (e.g., authentication):

```typescript
const message = 'Sign in to MyApp at ' + new Date().toISOString();
const signature = await wallet.signMessage(message);

// Verify on your server
console.log({
  message,
  signature: signature.signature,  // hex encoded
  publicKey: signature.publicKey,  // hex encoded
  address: signature.address
});
```

## Mobile Flow

### Deep Link Flow (Mobile-to-Mobile)

When your dApp runs in a mobile browser:

1. SDK generates a WalletLink session
2. Creates deep link: `https://wallet.sltn.io/connect?session=...`
3. Shows "Open Sultan Wallet" button
4. Auto-redirects after 500ms
5. Wallet shows approval screen
6. On approval, redirects back to your dApp

```typescript
const wallet = new SultanWalletSDK({
  qrContainerId: 'connect-area',
  autoRedirectMobile: true,
  onDeepLinkReady: (deepLink) => {
    // Custom handling if needed
    console.log('Wallet deep link:', deepLink);
  }
});
```

### QR Code Flow (Desktop)

When wallet is not installed on desktop:

1. SDK displays QR code
2. User scans with Sultan Wallet mobile app
3. Wallet prompts for approval
4. Connection established via relay

```html
<div id="wallet-connect-container">
  <!-- QR code will be rendered here -->
</div>
```

```typescript
const wallet = new SultanWalletSDK({
  qrContainerId: 'wallet-connect-container',
  onQRReady: (qrData) => {
    // QR code is now visible
    document.getElementById('status').textContent = 'Scan QR code with Sultan Wallet';
  }
});
```

## Session Persistence

```typescript
// Check if user was previously connected
const session = SultanWalletSDK.getStoredSession();
if (session) {
  // Restore session
  wallet.restoreSession(session);
  console.log('Reconnected:', session.address);
}

// Explicitly save session
wallet.on('connect', (account) => {
  SultanWalletSDK.storeSession(account);
});

// Clear session on disconnect
wallet.on('disconnect', () => {
  SultanWalletSDK.clearSession();
});
```

## Error Handling

```typescript
try {
  const account = await wallet.connect();
} catch (error) {
  if (error.code === 'USER_REJECTED') {
    console.log('User rejected connection');
  } else if (error.code === 'TIMEOUT') {
    console.log('Connection timed out');
  } else if (error.code === 'NETWORK_ERROR') {
    console.log('Network error - check internet connection');
  } else {
    console.error('Unknown error:', error);
  }
}
```

## Without SDK (Direct Integration)

If you don't want to use the SDK, you can integrate directly:

### Extension API

```typescript
// Check if extension is installed
if (window.sultanWallet) {
  // Request connection
  const account = await window.sultanWallet.connect();
  
  // Sign transaction
  const signed = await window.sultanWallet.signTransaction({
    to: 'sultan1...',
    amount: '1000000000'
  });
}
```

### WalletLink Protocol

```typescript
// 1. Generate session ID
const sessionId = crypto.randomUUID();

// 2. Connect to relay
const ws = new WebSocket('wss://relay.sltn.io');
ws.send(JSON.stringify({
  type: 'create_session',
  session_id: sessionId
}));

// 3. Display QR code with session link
const qrUrl = `https://wallet.sltn.io/connect?session=${sessionId}`;

// 4. Wait for wallet to connect
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'connected') {
    console.log('Wallet connected:', data.address);
  }
};
```

## Security Best Practices

1. **Validate addresses** - Always validate sultan1... addresses before use
2. **Handle disconnects** - Clear sensitive state when wallet disconnects
3. **HTTPS only** - Always use HTTPS for production dApps
4. **Don't store keys** - Never ask users to enter private keys

## Next Steps

- [Quick Start Guide](QUICK_START.md) - Basic integration
- [SDK Documentation](SDK.md) - Full SDK reference
- [API Reference](../api/API_REFERENCE.md) - Complete API documentation
