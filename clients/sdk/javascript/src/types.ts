// Core types for the vote system

export interface Vote {
  id: string;
  title: string;
  description: string;
  template_id: string;
  template_params: Record<string, any>;
  creator: string;
  created_at: string;
  commitment_start: string;
  commitment_end: string;
  reveal_start: string;
  reveal_end: string;
  status: VoteStatus;
  results?: VoteResults;
}

export enum VoteStatus {
  Created = 'Created',
  CommitmentPhase = 'CommitmentPhase',
  RevealPhase = 'RevealPhase',
  Completed = 'Completed',
  Cancelled = 'Cancelled'
}

export interface VoteConfig {
  title: string;
  description: string;
  template_id: string;
  template_params: Record<string, any>;
  commitment_duration_hours: number;
  reveal_duration_hours: number;
}

export interface Commitment {
  id: string;
  vote_id: string;
  voter: string;
  commitment_hash: string;
  salt: string;
  created_at: string;
}

export interface Reveal {
  id: string;
  vote_id: string;
  voter: string;
  value: any;
  salt: string;
  created_at: string;
}

export interface VoteResults {
  vote_id: string;
  total_votes: number;
  results: Record<string, any>;
  calculated_at: string;
}

export interface CommitRequest {
  voter: string;
  commitment_hash: string;
  salt: string;
}

export interface CommitResponse {
  commitment_id: string;
  success: boolean;
  message: string;
}

export interface RevealRequest {
  voter: string;
  value: any;
  salt: string;
}

export interface RevealResponse {
  reveal_id: string;
  success: boolean;
  message: string;
}

export interface ListQuery {
  page: number;
  page_size: number;
  status?: VoteStatus;
  template_id?: string;
  created_after?: string;
  created_before?: string;
}

export interface Page<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface VoteStats {
  vote_id: string;
  total_commitments: number;
  total_reveals: number;
  commitment_rate: number;
  reveal_rate: number;
  last_updated: string;
}

export interface VotePhase {
  vote_id: string;
  current_phase: string;
  phase_start: string;
  phase_end: string;
  time_remaining_seconds: number;
  progress_percentage: number;
}

// Error types
export class VoteError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: any
  ) {
    super(message);
    this.name = 'VoteError';
  }
}

export class NetworkError extends Error {
  constructor(
    message: string,
    public status?: number,
    public response?: any
  ) {
    super(message);
    this.name = 'NetworkError';
  }
}

// Configuration
export interface VoteSDKConfig {
  baseUrl: string;
  apiKey?: string;
  timeout?: number;
  retries?: number;
  websocketUrl?: string;
}

// WebSocket events
export interface WebSocketEvent {
  type: string;
  data: any;
  timestamp: string;
}

export interface VoteUpdateEvent extends WebSocketEvent {
  type: 'vote_update';
  data: Vote;
}

export interface CommitmentEvent extends WebSocketEvent {
  type: 'commitment';
  data: Commitment;
}

export interface RevealEvent extends WebSocketEvent {
  type: 'reveal';
  data: Reveal;
}

export interface ResultsEvent extends WebSocketEvent {
  type: 'results';
  data: VoteResults;
}
