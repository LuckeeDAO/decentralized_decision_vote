"""
Luckee DAO Vote SDK - Python

A Python SDK for interacting with the Luckee DAO Decentralized Decision Vote System.
"""

from .client import VoteClient, create_vote_client
from .types import (
    Vote,
    VoteConfig,
    VoteStatus,
    Commitment,
    Reveal,
    VoteResults,
    VoteStats,
    VotePhase,
    CommitRequest,
    CommitResponse,
    RevealRequest,
    RevealResponse,
    ListQuery,
    Page,
    VoteError,
    NetworkError,
    WebSocketEvent,
    VoteUpdateEvent,
    CommitmentEvent,
    RevealEvent,
    ResultsEvent,
)
from .crypto import (
    generate_salt,
    generate_id,
    create_commitment,
    verify_commitment,
    create_vote_commitment,
    verify_vote_commitment,
)

__version__ = "0.1.0"
__author__ = "Luckee DAO Team"
__email__ = "team@luckee-dao.com"

__all__ = [
    # Client
    "VoteClient",
    "create_vote_client",
    # Types
    "Vote",
    "VoteConfig",
    "VoteStatus",
    "Commitment",
    "Reveal",
    "VoteResults",
    "VoteStats",
    "VotePhase",
    "CommitRequest",
    "CommitResponse",
    "RevealRequest",
    "RevealResponse",
    "ListQuery",
    "Page",
    "VoteError",
    "NetworkError",
    "WebSocketEvent",
    "VoteUpdateEvent",
    "CommitmentEvent",
    "RevealEvent",
    "ResultsEvent",
    # Crypto
    "generate_salt",
    "generate_id",
    "create_commitment",
    "verify_commitment",
    "create_vote_commitment",
    "verify_vote_commitment",
]
