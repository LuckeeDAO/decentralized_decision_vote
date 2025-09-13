# Luckee DAO Vote SDK - Python

A Python SDK for interacting with the Luckee DAO Decentralized Decision Vote System.

## Installation

```bash
pip install luckee-dao-vote-sdk
```

## Usage

### Basic Setup

```python
import asyncio
from luckee_dao_vote_sdk import VoteClient, VoteConfig, VoteStatus, create_vote_commitment

async def main():
    client = VoteClient({
        "base_url": "https://api.luckee-dao.com",
        "api_key": "your-api-key",  # optional
        "timeout": 30,
        "retries": 3,
        "websocket_url": "wss://api.luckee-dao.com/ws"  # optional
    })
    
    # Use the client...
    await client.close()

# Run the async function
asyncio.run(main())
```

### Creating a Vote

```python
from luckee_dao_vote_sdk import VoteConfig

vote_config = VoteConfig(
    title="Should we implement feature X?",
    description="This vote will decide whether to implement the new feature",
    template_id="yes_no",
    template_params={
        "options": ["yes", "no"]
    },
    commitment_duration_hours=24,
    reveal_duration_hours=24
)

vote_id = await client.create_vote(vote_config)
print(f"Created vote: {vote_id}")
```

### Committing a Vote

```python
from luckee_dao_vote_sdk import create_vote_commitment

# Create a commitment for your vote
commitment_hash, salt = create_vote_commitment("yes", "voter123")

# Submit the commitment
response = await client.commit_vote(vote_id, {
    "voter": "voter123",
    "commitment_hash": commitment_hash,
    "salt": salt
})

print(f"Commitment submitted: {response.commitment_id}")
```

### Revealing a Vote

```python
# Submit the reveal
reveal_response = await client.reveal_vote(vote_id, {
    "voter": "voter123",
    "value": "yes",
    "salt": salt  # Same salt used in commitment
})

print(f"Vote revealed: {reveal_response.reveal_id}")
```

### Getting Results

```python
results = await client.get_results(vote_id)
print(f"Vote results: {results}")
```

### Real-time Updates with WebSocket

```python
# Connect to WebSocket
await client.connect_websocket()

# Listen for vote updates
def on_vote_update(event):
    print(f"Vote updated: {event.data}")

client.on("vote_update", on_vote_update)

# Listen for new commitments
def on_commitment(event):
    print(f"New commitment: {event.data}")

client.on("commitment", on_commitment)

# Listen for reveals
def on_reveal(event):
    print(f"New reveal: {event.data}")

client.on("reveal", on_reveal)

# Listen for results
def on_results(event):
    print(f"Vote results: {event.data}")

client.on("results", on_results)

# Keep the connection alive
await asyncio.sleep(60)  # Listen for 60 seconds
await client.disconnect_websocket()
```

### Listing Votes

```python
from luckee_dao_vote_sdk import ListQuery, VoteStatus

votes = await client.list_votes(ListQuery(
    page=1,
    page_size=10,
    status=VoteStatus.COMMITMENT_PHASE
))

print(f"Votes: {votes.items}")
```

### Error Handling

```python
from luckee_dao_vote_sdk import VoteError, NetworkError

try:
    vote = await client.get_vote("invalid-id")
except VoteError as e:
    print(f"Vote error: {e.message}")
    print(f"Error code: {e.code}")
except NetworkError as e:
    print(f"Network error: {e.message}")
    print(f"Status: {e.status}")
```

### Using Context Manager

```python
async def main():
    async with VoteClient(config) as client:
        vote_id = await client.create_vote(vote_config)
        # Client will be automatically closed when exiting the context
```

## API Reference

### VoteClient

Main client class for interacting with the vote system.

#### Constructor

```python
VoteClient(config: VoteSDKConfig)
```

#### Methods

- `create_vote(config: VoteConfig) -> str`
- `get_vote(vote_id: str) -> Vote`
- `list_votes(query: ListQuery) -> Page[Vote]`
- `commit_vote(vote_id: str, request: CommitRequest) -> CommitResponse`
- `reveal_vote(vote_id: str, request: RevealRequest) -> RevealResponse`
- `get_results(vote_id: str) -> VoteResults`
- `get_vote_stats(vote_id: str) -> VoteStats`
- `get_vote_phase(vote_id: str) -> VotePhase`
- `connect_websocket() -> None`
- `disconnect_websocket() -> None`
- `on(event_type: str, handler: Callable) -> None`
- `off(event_type: str, handler: Callable) -> None`
- `health_check() -> bool`

### Crypto Utilities

- `generate_salt() -> str`
- `generate_id() -> str`
- `create_commitment(value: str, salt: str) -> str`
- `verify_commitment(value: str, salt: str, commitment_hash: str) -> bool`
- `create_vote_commitment(value: str, voter: str) -> Tuple[str, str]`
- `verify_vote_commitment(value: str, salt: str, commitment_hash: str) -> bool`

## Types

The SDK provides comprehensive type definitions using Pydantic:

- `Vote`, `VoteConfig`, `VoteStatus`
- `Commitment`, `Reveal`
- `VoteResults`, `VoteStats`, `VotePhase`
- `CommitRequest`, `CommitResponse`
- `RevealRequest`, `RevealResponse`
- `ListQuery`, `Page[T]`
- `VoteError`, `NetworkError`
- `WebSocketEvent` and related event types

## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/luckee-dao/decentralized_decision_vote.git
cd decentralized_decision_vote/clients/sdk/python

# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run linting
flake8 src/
black src/
isort src/

# Run type checking
mypy src/
```

### Testing

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src/luckee_dao_vote_sdk

# Run specific test file
pytest tests/test_client.py

# Run with verbose output
pytest -v
```

## License

MIT
