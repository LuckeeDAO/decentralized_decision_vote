# Luckee DAO Vote SDK - JavaScript/TypeScript

A TypeScript/JavaScript SDK for interacting with the Luckee DAO Decentralized Decision Vote System.

## Installation

```bash
npm install @luckee-dao/vote-sdk
```

## Usage

### Basic Setup

```typescript
import { VoteClient, VoteConfig, VoteStatus } from '@luckee-dao/vote-sdk';

const client = new VoteClient({
  baseUrl: 'https://api.luckee-dao.com',
  apiKey: 'your-api-key', // optional
  timeout: 30000,
  retries: 3,
  websocketUrl: 'wss://api.luckee-dao.com/ws' // optional
});
```

### Creating a Vote

```typescript
const voteConfig: VoteConfig = {
  title: 'Should we implement feature X?',
  description: 'This vote will decide whether to implement the new feature',
  template_id: 'yes_no',
  template_params: {
    options: ['yes', 'no']
  },
  commitment_duration_hours: 24,
  reveal_duration_hours: 24
};

const voteId = await client.createVote(voteConfig);
console.log('Created vote:', voteId);
```

### Committing a Vote

```typescript
import { createVoteCommitment } from '@luckee-dao/vote-sdk';

// Create a commitment for your vote
const { commitmentHash, salt } = createVoteCommitment('yes', 'voter123');

// Submit the commitment
const response = await client.commitVote(voteId, {
  voter: 'voter123',
  commitment_hash: commitmentHash,
  salt: salt
});

console.log('Commitment submitted:', response.commitment_id);
```

### Revealing a Vote

```typescript
// Submit the reveal
const revealResponse = await client.revealVote(voteId, {
  voter: 'voter123',
  value: 'yes',
  salt: salt // Same salt used in commitment
});

console.log('Vote revealed:', revealResponse.reveal_id);
```

### Getting Results

```typescript
const results = await client.getResults(voteId);
console.log('Vote results:', results);
```

### Real-time Updates with WebSocket

```typescript
// Connect to WebSocket
await client.connectWebSocket();

// Listen for vote updates
client.on('vote_update', (event) => {
  console.log('Vote updated:', event.data);
});

// Listen for new commitments
client.on('commitment', (event) => {
  console.log('New commitment:', event.data);
});

// Listen for reveals
client.on('reveal', (event) => {
  console.log('New reveal:', event.data);
});

// Listen for results
client.on('results', (event) => {
  console.log('Vote results:', event.data);
});
```

### Listing Votes

```typescript
const votes = await client.listVotes({
  page: 1,
  page_size: 10,
  status: VoteStatus.CommitmentPhase
});

console.log('Votes:', votes.items);
```

### Error Handling

```typescript
try {
  const vote = await client.getVote('invalid-id');
} catch (error) {
  if (error instanceof VoteError) {
    console.error('Vote error:', error.message);
    console.error('Error code:', error.code);
  } else if (error instanceof NetworkError) {
    console.error('Network error:', error.message);
    console.error('Status:', error.status);
  }
}
```

## API Reference

### VoteClient

Main client class for interacting with the vote system.

#### Constructor

```typescript
new VoteClient(config: VoteSDKConfig)
```

#### Methods

- `createVote(config: VoteConfig): Promise<string>`
- `getVote(voteId: string): Promise<Vote>`
- `listVotes(query: ListQuery): Promise<Page<Vote>>`
- `commitVote(voteId: string, request: CommitRequest): Promise<CommitResponse>`
- `revealVote(voteId: string, request: RevealRequest): Promise<RevealResponse>`
- `getResults(voteId: string): Promise<VoteResults>`
- `getVoteStats(voteId: string): Promise<VoteStats>`
- `getVotePhase(voteId: string): Promise<VotePhase>`
- `connectWebSocket(): Promise<void>`
- `disconnectWebSocket(): void`
- `on(eventType: string, handler: (event: WebSocketEvent) => void): void`
- `off(eventType: string, handler: (event: WebSocketEvent) => void): void`
- `healthCheck(): Promise<boolean>`

### Crypto Utilities

- `generateSalt(): string`
- `generateId(): string`
- `createCommitment(value: string, salt: string): string`
- `verifyCommitment(value: string, salt: string, commitmentHash: string): boolean`
- `createVoteCommitment(value: string, voter: string): { commitmentHash: string; salt: string }`
- `verifyVoteCommitment(value: string, salt: string, commitmentHash: string): boolean`

## Types

The SDK provides comprehensive TypeScript types for all data structures:

- `Vote`, `VoteConfig`, `VoteStatus`
- `Commitment`, `Reveal`
- `VoteResults`, `VoteStats`, `VotePhase`
- `CommitRequest`, `CommitResponse`
- `RevealRequest`, `RevealResponse`
- `ListQuery`, `Page<T>`
- `VoteError`, `NetworkError`
- `WebSocketEvent` and related event types

## Browser Support

The SDK works in both Node.js and browser environments. For browser usage, make sure to include the necessary polyfills for WebSocket and crypto functionality.

## License

MIT
