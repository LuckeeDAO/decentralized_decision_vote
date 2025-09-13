"""
Cryptographic utilities for the Luckee DAO Vote SDK.
"""

import hashlib
import secrets
from typing import Tuple


def generate_salt() -> str:
    """Generate a random salt for commitment schemes."""
    return secrets.token_hex(32)


def generate_id() -> str:
    """Generate a unique ID."""
    return secrets.token_hex(16)


def create_commitment(value: str, salt: str) -> str:
    """Create a commitment hash using SHA-256."""
    combined = f"{value}:{salt}"
    return hashlib.sha256(combined.encode()).hexdigest()


def verify_commitment(value: str, salt: str, commitment_hash: str) -> bool:
    """Verify a commitment against a revealed value."""
    expected_hash = create_commitment(value, salt)
    return expected_hash == commitment_hash


def create_vote_commitment(value: str, voter: str) -> Tuple[str, str]:
    """Create a commitment for a vote value.
    
    Returns:
        Tuple of (commitment_hash, salt)
    """
    salt = generate_salt()
    commitment_hash = create_commitment(value, salt)
    return commitment_hash, salt


def verify_vote_commitment(value: str, salt: str, commitment_hash: str) -> bool:
    """Verify a vote commitment."""
    return verify_commitment(value, salt, commitment_hash)


def hash_string(input_string: str) -> str:
    """Hash a string using SHA-256."""
    return hashlib.sha256(input_string.encode()).hexdigest()


def generate_secure_random(length: int = 32) -> str:
    """Generate a secure random string."""
    return secrets.token_hex(length)


def is_valid_hex(input_string: str) -> bool:
    """Validate that a string is a valid hex string."""
    try:
        int(input_string, 16)
        return True
    except ValueError:
        return False


def is_valid_commitment_hash(hash_string: str) -> bool:
    """Validate that a string is a valid commitment hash."""
    return is_valid_hex(hash_string) and len(hash_string) == 64  # SHA-256 produces 64 hex characters


def is_valid_salt(salt: str) -> bool:
    """Validate that a string is a valid salt."""
    return is_valid_hex(salt) and len(salt) == 64  # 32 bytes = 64 hex characters
