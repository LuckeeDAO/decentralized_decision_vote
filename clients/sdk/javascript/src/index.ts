// Main exports for the Vote SDK
export * from './types';
export * from './client';
export * from './crypto';

// Re-export commonly used items for convenience
export { VoteClient, createVoteClient } from './client';
export { 
  generateSalt, 
  generateId, 
  createCommitment, 
  verifyCommitment,
  createVoteCommitment,
  verifyVoteCommitment 
} from './crypto';

// Default export
export { VoteClient as default } from './client';
