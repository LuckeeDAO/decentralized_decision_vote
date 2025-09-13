"""
Type definitions for the Luckee DAO Vote SDK.
"""

from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional, Union
from pydantic import BaseModel, Field


class VoteStatus(str, Enum):
    """Vote status enumeration."""
    CREATED = "Created"
    COMMITMENT_PHASE = "CommitmentPhase"
    REVEAL_PHASE = "RevealPhase"
    COMPLETED = "Completed"
    CANCELLED = "Cancelled"


class Vote(BaseModel):
    """Vote data structure."""
    id: str
    title: str
    description: str
    template_id: str
    template_params: Dict[str, Any]
    creator: str
    created_at: datetime
    commitment_start: datetime
    commitment_end: datetime
    reveal_start: datetime
    reveal_end: datetime
    status: VoteStatus
    results: Optional["VoteResults"] = None


class VoteConfig(BaseModel):
    """Configuration for creating a new vote."""
    title: str
    description: str
    template_id: str
    template_params: Dict[str, Any]
    commitment_duration_hours: int
    reveal_duration_hours: int


class Commitment(BaseModel):
    """Commitment data structure."""
    id: str
    vote_id: str
    voter: str
    commitment_hash: str
    salt: str
    created_at: datetime


class Reveal(BaseModel):
    """Reveal data structure."""
    id: str
    vote_id: str
    voter: str
    value: Any
    salt: str
    created_at: datetime


class VoteResults(BaseModel):
    """Vote results data structure."""
    vote_id: str
    total_votes: int
    results: Dict[str, Any]
    calculated_at: datetime


class CommitRequest(BaseModel):
    """Request to commit a vote."""
    voter: str
    commitment_hash: str
    salt: str


class CommitResponse(BaseModel):
    """Response from committing a vote."""
    commitment_id: str
    success: bool
    message: str


class RevealRequest(BaseModel):
    """Request to reveal a vote."""
    voter: str
    value: Any
    salt: str


class RevealResponse(BaseModel):
    """Response from revealing a vote."""
    reveal_id: str
    success: bool
    message: str


class ListQuery(BaseModel):
    """Query parameters for listing votes."""
    page: int = Field(ge=1, default=1)
    page_size: int = Field(ge=1, le=100, default=10)
    status: Optional[VoteStatus] = None
    template_id: Optional[str] = None
    created_after: Optional[datetime] = None
    created_before: Optional[datetime] = None


class Page(BaseModel):
    """Paginated response data structure."""
    items: List[Any]
    total: int
    page: int
    page_size: int
    total_pages: int


class VoteStats(BaseModel):
    """Vote statistics."""
    vote_id: str
    total_commitments: int
    total_reveals: int
    commitment_rate: float
    reveal_rate: float
    last_updated: datetime


class VotePhase(BaseModel):
    """Vote phase information."""
    vote_id: str
    current_phase: str
    phase_start: datetime
    phase_end: datetime
    time_remaining_seconds: int
    progress_percentage: float


class VoteSDKConfig(BaseModel):
    """Configuration for the Vote SDK."""
    base_url: str
    api_key: Optional[str] = None
    timeout: int = Field(ge=1, default=30)
    retries: int = Field(ge=0, default=3)
    websocket_url: Optional[str] = None


class WebSocketEvent(BaseModel):
    """Base WebSocket event."""
    type: str
    data: Any
    timestamp: datetime


class VoteUpdateEvent(WebSocketEvent):
    """Vote update WebSocket event."""
    type: str = "vote_update"
    data: Vote


class CommitmentEvent(WebSocketEvent):
    """Commitment WebSocket event."""
    type: str = "commitment"
    data: Commitment


class RevealEvent(WebSocketEvent):
    """Reveal WebSocket event."""
    type: str = "reveal"
    data: Reveal


class ResultsEvent(WebSocketEvent):
    """Results WebSocket event."""
    type: str = "results"
    data: VoteResults


class VoteError(Exception):
    """Custom exception for vote-related errors."""
    
    def __init__(
        self,
        message: str,
        code: Optional[str] = None,
        details: Optional[Dict[str, Any]] = None
    ):
        super().__init__(message)
        self.message = message
        self.code = code
        self.details = details or {}


class NetworkError(Exception):
    """Custom exception for network-related errors."""
    
    def __init__(
        self,
        message: str,
        status: Optional[int] = None,
        response: Optional[Dict[str, Any]] = None
    ):
        super().__init__(message)
        self.message = message
        self.status = status
        self.response = response or {}
