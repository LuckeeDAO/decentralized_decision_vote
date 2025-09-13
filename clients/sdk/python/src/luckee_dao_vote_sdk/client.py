"""
Main client for the Luckee DAO Vote SDK.
"""

import asyncio
import json
import logging
from typing import Any, Callable, Dict, List, Optional
import httpx
import websockets
from websockets.exceptions import ConnectionClosed

from .types import (
    Vote,
    VoteConfig,
    VoteResults,
    VoteStats,
    VotePhase,
    ListQuery,
    Page,
    CommitRequest,
    CommitResponse,
    RevealRequest,
    RevealResponse,
    VoteSDKConfig,
    VoteError,
    NetworkError,
    WebSocketEvent,
    VoteUpdateEvent,
    CommitmentEvent,
    RevealEvent,
    ResultsEvent,
)

logger = logging.getLogger(__name__)


class VoteClient:
    """Main client for interacting with the vote system."""
    
    def __init__(self, config: VoteSDKConfig):
        """Initialize the vote client."""
        self.config = config
        self.http_client = httpx.AsyncClient(
            base_url=config.base_url,
            timeout=config.timeout,
            headers={
                "Content-Type": "application/json",
                **({"Authorization": f"Bearer {config.api_key}"} if config.api_key else {})
            }
        )
        self.websocket: Optional[websockets.WebSocketServerProtocol] = None
        self.event_handlers: Dict[str, List[Callable[[WebSocketEvent], None]]] = {}
        self._websocket_task: Optional[asyncio.Task] = None
    
    async def __aenter__(self):
        """Async context manager entry."""
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
    
    async def close(self):
        """Close the client and cleanup resources."""
        if self.websocket:
            await self.websocket.close()
        await self.http_client.aclose()
    
    # Vote management methods
    
    async def create_vote(self, config: VoteConfig) -> str:
        """Create a new vote."""
        try:
            response = await self.http_client.post("/api/v1/votes", json=config.dict())
            response.raise_for_status()
            return response.json()["vote_id"]
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to create vote: {str(e)}")
    
    async def get_vote(self, vote_id: str) -> Vote:
        """Get a vote by ID."""
        try:
            response = await self.http_client.get(f"/api/v1/votes/{vote_id}")
            response.raise_for_status()
            return Vote(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to get vote: {str(e)}")
    
    async def list_votes(self, query: ListQuery) -> Page[Vote]:
        """List votes with pagination."""
        try:
            params = {k: v for k, v in query.dict().items() if v is not None}
            response = await self.http_client.get("/api/v1/votes", params=params)
            response.raise_for_status()
            data = response.json()
            return Page(
                items=[Vote(**item) for item in data["items"]],
                total=data["total"],
                page=data["page"],
                page_size=data["page_size"],
                total_pages=data["total_pages"]
            )
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to list votes: {str(e)}")
    
    async def get_vote_stats(self, vote_id: str) -> VoteStats:
        """Get vote statistics."""
        try:
            response = await self.http_client.get(f"/api/v1/votes/{vote_id}/stats")
            response.raise_for_status()
            return VoteStats(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to get vote stats: {str(e)}")
    
    async def get_vote_phase(self, vote_id: str) -> VotePhase:
        """Get vote phase information."""
        try:
            response = await self.http_client.get(f"/api/v1/votes/{vote_id}/phase")
            response.raise_for_status()
            return VotePhase(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to get vote phase: {str(e)}")
    
    # Commitment operations
    
    async def commit_vote(self, vote_id: str, request: CommitRequest) -> CommitResponse:
        """Commit a vote."""
        try:
            response = await self.http_client.post(
                f"/api/v1/votes/{vote_id}/commit",
                json=request.dict()
            )
            response.raise_for_status()
            return CommitResponse(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to commit vote: {str(e)}")
    
    # Reveal operations
    
    async def reveal_vote(self, vote_id: str, request: RevealRequest) -> RevealResponse:
        """Reveal a vote."""
        try:
            response = await self.http_client.post(
                f"/api/v1/votes/{vote_id}/reveal",
                json=request.dict()
            )
            response.raise_for_status()
            return RevealResponse(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to reveal vote: {str(e)}")
    
    # Results
    
    async def get_results(self, vote_id: str) -> VoteResults:
        """Get vote results."""
        try:
            response = await self.http_client.get(f"/api/v1/votes/{vote_id}/results")
            response.raise_for_status()
            return VoteResults(**response.json())
        except httpx.HTTPStatusError as e:
            raise self._handle_http_error(e)
        except Exception as e:
            raise VoteError(f"Failed to get results: {str(e)}")
    
    # WebSocket operations
    
    async def connect_websocket(self) -> None:
        """Connect to WebSocket for real-time updates."""
        if not self.config.websocket_url:
            raise VoteError("WebSocket URL not configured")
        
        try:
            self.websocket = await websockets.connect(self.config.websocket_url)
            self._websocket_task = asyncio.create_task(self._websocket_listener())
            logger.info("WebSocket connected")
        except Exception as e:
            raise VoteError(f"Failed to connect WebSocket: {str(e)}")
    
    async def disconnect_websocket(self) -> None:
        """Disconnect from WebSocket."""
        if self._websocket_task:
            self._websocket_task.cancel()
            try:
                await self._websocket_task
            except asyncio.CancelledError:
                pass
        
        if self.websocket:
            await self.websocket.close()
            self.websocket = None
        
        logger.info("WebSocket disconnected")
    
    async def _websocket_listener(self) -> None:
        """Listen for WebSocket messages."""
        try:
            async for message in self.websocket:
                try:
                    data = json.loads(message)
                    event = WebSocketEvent(**data)
                    await self._handle_websocket_event(event)
                except Exception as e:
                    logger.error(f"Failed to process WebSocket message: {e}")
        except ConnectionClosed:
            logger.info("WebSocket connection closed")
        except Exception as e:
            logger.error(f"WebSocket error: {e}")
    
    async def _handle_websocket_event(self, event: WebSocketEvent) -> None:
        """Handle incoming WebSocket events."""
        handlers = self.event_handlers.get(event.type, [])
        for handler in handlers:
            try:
                if asyncio.iscoroutinefunction(handler):
                    await handler(event)
                else:
                    handler(event)
            except Exception as e:
                logger.error(f"Error in event handler for {event.type}: {e}")
    
    # Event handling
    
    def on(self, event_type: str, handler: Callable[[WebSocketEvent], None]) -> None:
        """Register an event handler."""
        if event_type not in self.event_handlers:
            self.event_handlers[event_type] = []
        self.event_handlers[event_type].append(handler)
    
    def off(self, event_type: str, handler: Callable[[WebSocketEvent], None]) -> None:
        """Unregister an event handler."""
        if event_type in self.event_handlers:
            try:
                self.event_handlers[event_type].remove(handler)
            except ValueError:
                pass
    
    # Utility methods
    
    def _handle_http_error(self, error: httpx.HTTPStatusError) -> Exception:
        """Handle HTTP errors."""
        try:
            error_data = error.response.json()
            message = error_data.get("message", "HTTP request failed")
            code = error_data.get("code")
            details = error_data.get("details", {})
            return VoteError(message, code, details)
        except:
            return NetworkError(
                f"HTTP {error.response.status_code}: {error.response.text}",
                error.response.status_code,
                {"response": error.response.text}
            )
    
    async def health_check(self) -> bool:
        """Check if the service is healthy."""
        try:
            response = await self.http_client.get("/health")
            return response.status_code == 200
        except:
            return False


def create_vote_client(config: VoteSDKConfig) -> VoteClient:
    """Factory function to create a vote client."""
    return VoteClient(config)
