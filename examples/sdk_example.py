#!/usr/bin/env python3
"""
Sultan L1 SDK Example - Python

This example demonstrates how to interact with Sultan L1 blockchain using Python.

Installation:
    pip install pynacl bech32 requests

Usage:
    python sdk_example.py
"""

import hashlib
import json
import time
from typing import Optional, Tuple

import requests
from bech32 import bech32_encode, convertbits
from nacl.signing import SigningKey
from nacl.encoding import RawEncoder

RPC_URL = "https://rpc.sltn.io"


# ============================================================================
# WALLET
# ============================================================================

class SultanWallet:
    """Sultan L1 Wallet for signing transactions."""
    
    def __init__(self, private_key_hex: Optional[str] = None):
        """
        Create a new wallet or import from private key.
        
        Args:
            private_key_hex: Optional hex-encoded private key. If None, generates new key.
        """
        if private_key_hex:
            self.signing_key = SigningKey(bytes.fromhex(private_key_hex))
        else:
            self.signing_key = SigningKey.generate()
        
        self.public_key = self.signing_key.verify_key.encode()
        self.address = self._derive_address()
    
    def _derive_address(self) -> str:
        """Derive bech32 address from public key."""
        # SHA-256 hash of public key, take first 20 bytes
        hash_bytes = hashlib.sha256(self.public_key).digest()[:20]
        # Bech32 encode with "sultan" prefix
        data = convertbits(list(hash_bytes), 8, 5)
        return bech32_encode("sultan", data)
    
    def sign(self, message: dict) -> Tuple[str, str]:
        """
        Sign a message dictionary.
        
        CRITICAL: Uses sorted keys for deterministic JSON ordering.
        
        Args:
            message: Dictionary to sign
            
        Returns:
            Tuple of (signature_hex, public_key_hex)
        """
        # CRITICAL: Use sorted keys for deterministic JSON
        msg_bytes = json.dumps(message, sort_keys=True).encode()
        signed = self.signing_key.sign(msg_bytes, encoder=RawEncoder)
        
        # IMPORTANT: Use hex encoding, NOT base64
        signature = signed.signature.hex()
        pubkey = self.public_key.hex()
        
        return signature, pubkey
    
    @property
    def private_key_hex(self) -> str:
        """Get private key as hex string (keep secret!)."""
        return self.signing_key.encode().hex()
    
    @property
    def public_key_hex(self) -> str:
        """Get public key as hex string."""
        return self.public_key.hex()


# ============================================================================
# SDK CLIENT
# ============================================================================

class SultanSDK:
    """Sultan L1 SDK Client."""
    
    def __init__(self, rpc_url: str = RPC_URL):
        """
        Initialize SDK client.
        
        Args:
            rpc_url: RPC endpoint URL
        """
        self.rpc_url = rpc_url
    
    @classmethod
    def mainnet(cls) -> "SultanSDK":
        """Create SDK instance for mainnet."""
        return cls("https://rpc.sltn.io")
    
    @classmethod
    def testnet(cls) -> "SultanSDK":
        """Create SDK instance for testnet."""
        return cls("https://testnet.sltn.io")
    
    def get_status(self) -> dict:
        """Get network status."""
        res = requests.get(f"{self.rpc_url}/status")
        res.raise_for_status()
        return res.json()
    
    def get_balance(self, address: str) -> dict:
        """
        Get balance for an address.
        
        Args:
            address: Sultan address (sultan1...)
            
        Returns:
            Dict with 'address', 'balance' (atomic units), 'nonce'
        """
        res = requests.get(f"{self.rpc_url}/balance/{address}")
        res.raise_for_status()
        return res.json()
    
    def get_balance_sltn(self, address: str) -> float:
        """
        Get balance in human-readable SLTN.
        
        Args:
            address: Sultan address (sultan1...)
            
        Returns:
            Balance in SLTN
        """
        data = self.get_balance(address)
        return data["balance"] / 1e9
    
    def send_sltn(
        self,
        wallet: SultanWallet,
        to: str,
        amount_sltn: float,
        memo: str = ""
    ) -> dict:
        """
        Send SLTN tokens.
        
        CRITICAL: This method demonstrates proper signature format:
        - Keys are alphabetically sorted (json.dumps with sort_keys=True)
        - Amount is a STRING in the signed message
        - Signature and public key are hex encoded (NOT base64)
        
        Args:
            wallet: Sender wallet
            to: Recipient address
            amount_sltn: Amount in SLTN
            memo: Optional memo
            
        Returns:
            Transaction result dict
        """
        # Get current nonce
        balance_info = self.get_balance(wallet.address)
        nonce = balance_info["nonce"]
        
        # Convert to atomic units
        amount_atomic = int(amount_sltn * 1e9)
        timestamp = int(time.time())
        
        # Create message for signing
        # CRITICAL: Amount must be a STRING
        tx_for_signing = {
            "amount": str(amount_atomic),  # MUST be string
            "from": wallet.address,
            "memo": memo,
            "nonce": nonce,
            "timestamp": timestamp,
            "to": to
        }
        
        # Sign with sorted keys
        signature, pubkey = wallet.sign(tx_for_signing)
        
        # Build request body
        request = {
            "tx": {
                "from": wallet.address,
                "to": to,
                "amount": amount_atomic,  # Can be int in request body
                "timestamp": timestamp,
                "nonce": nonce,
                "memo": memo
            },
            "signature": signature,    # Hex encoded (128 chars)
            "public_key": pubkey       # Hex encoded (64 chars)
        }
        
        # Submit transaction
        res = requests.post(
            f"{self.rpc_url}/tx",
            json=request,
            headers={"Content-Type": "application/json"}
        )
        res.raise_for_status()
        return res.json()
    
    def get_transaction(self, tx_hash: str) -> dict:
        """
        Get transaction by hash.
        
        Args:
            tx_hash: Transaction hash
            
        Returns:
            Transaction details
        """
        res = requests.get(f"{self.rpc_url}/tx/{tx_hash}")
        res.raise_for_status()
        return res.json()
    
    def wait_for_confirmation(
        self,
        tx_hash: str,
        timeout_seconds: int = 30
    ) -> dict:
        """
        Wait for transaction confirmation.
        
        Args:
            tx_hash: Transaction hash
            timeout_seconds: Maximum wait time
            
        Returns:
            Confirmed transaction
            
        Raises:
            TimeoutError: If transaction not confirmed in time
        """
        start = time.time()
        
        while time.time() - start < timeout_seconds:
            tx = self.get_transaction(tx_hash)
            if tx.get("status") == "confirmed":
                return tx
            time.sleep(2)  # Wait one block time
        
        raise TimeoutError("Transaction confirmation timeout")


# ============================================================================
# EXAMPLE USAGE
# ============================================================================

def main():
    print("=== Sultan L1 SDK Example (Python) ===\n")
    
    # Initialize SDK
    sdk = SultanSDK.mainnet()
    
    # Check network status
    print("📡 Checking network status...")
    status = sdk.get_status()
    print(f"   Block Height: {status['block_height']}")
    print(f"   Validators: {status['validators']}")
    print(f"   TPS Capacity: {status['tps_capacity']}")
    print(f"   Version: {status['version']}\n")
    
    # Create a new wallet
    print("🔑 Creating new wallet...")
    wallet = SultanWallet()
    print(f"   Address: {wallet.address}")
    print(f"   Public Key: {wallet.public_key_hex[:16]}...\n")
    
    # Check balance
    print("💰 Checking balance...")
    balance = sdk.get_balance_sltn(wallet.address)
    print(f"   Balance: {balance} SLTN\n")
    
    # Example: Send transaction (commented out - requires funded wallet)
    # print("📤 Sending 10 SLTN...")
    # tx = sdk.send_sltn(wallet, "sultan1recipient...", 10.0)
    # print(f"   TX Hash: {tx['hash']}")
    #
    # confirmed = sdk.wait_for_confirmation(tx['hash'])
    # print(f"   Status: {confirmed['status']}")
    
    print("✅ SDK example complete!")
    print("\nFor more examples, see: https://github.com/Sultan-Labs/DOCS/tree/main/examples")


if __name__ == "__main__":
    main()
