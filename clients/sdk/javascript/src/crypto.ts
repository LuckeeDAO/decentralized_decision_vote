import { createHash, randomBytes } from 'crypto';

/**
 * Generate a random salt for commitment schemes
 */
export function generateSalt(): string {
  return randomBytes(32).toString('hex');
}

/**
 * Generate a unique ID
 */
export function generateId(): string {
  return randomBytes(16).toString('hex');
}

/**
 * Create a commitment hash using SHA-256
 */
export function createCommitment(value: string, salt: string): string {
  const combined = `${value}:${salt}`;
  return createHash('sha256').update(combined).digest('hex');
}

/**
 * Verify a commitment against a revealed value
 */
export function verifyCommitment(value: string, salt: string, commitmentHash: string): boolean {
  const expectedHash = createCommitment(value, salt);
  return expectedHash === commitmentHash;
}

/**
 * Create a commitment for a vote value
 */
export function createVoteCommitment(value: string, voter: string): { commitmentHash: string; salt: string } {
  const salt = generateSalt();
  const commitmentHash = createCommitment(value, salt);
  return { commitmentHash, salt };
}

/**
 * Verify a vote commitment
 */
export function verifyVoteCommitment(value: string, salt: string, commitmentHash: string): boolean {
  return verifyCommitment(value, salt, commitmentHash);
}

/**
 * Hash a string using SHA-256
 */
export function hashString(input: string): string {
  return createHash('sha256').update(input).digest('hex');
}

/**
 * Generate a secure random string
 */
export function generateSecureRandom(length: number = 32): string {
  return randomBytes(length).toString('hex');
}

/**
 * Validate that a string is a valid hex string
 */
export function isValidHex(input: string): boolean {
  return /^[0-9a-fA-F]+$/.test(input);
}

/**
 * Validate that a string is a valid commitment hash
 */
export function isValidCommitmentHash(hash: string): boolean {
  return isValidHex(hash) && hash.length === 64; // SHA-256 produces 64 hex characters
}

/**
 * Validate that a string is a valid salt
 */
export function isValidSalt(salt: string): boolean {
  return isValidHex(salt) && salt.length === 64; // 32 bytes = 64 hex characters
}
