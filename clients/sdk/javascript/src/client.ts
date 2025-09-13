import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import WebSocket from 'ws';
import {
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
  ResultsEvent
} from './types';

export class VoteClient {
  private httpClient: AxiosInstance;
  private wsClient?: WebSocket;
  private config: VoteSDKConfig;
  private eventHandlers: Map<string, ((event: WebSocketEvent) => void)[]> = new Map();

  constructor(config: VoteSDKConfig) {
    this.config = {
      timeout: 30000,
      retries: 3,
      ...config
    };

    this.httpClient = axios.create({
      baseURL: this.config.baseUrl,
      timeout: this.config.timeout,
      headers: {
        'Content-Type': 'application/json',
        ...(this.config.apiKey && { 'Authorization': `Bearer ${this.config.apiKey}` })
      }
    });

    // Add request interceptor for retries
    this.httpClient.interceptors.request.use(
      (config) => config,
      (error) => Promise.reject(error)
    );

    // Add response interceptor for error handling
    this.httpClient.interceptors.response.use(
      (response) => response,
      async (error) => {
        if (error.response?.status >= 500 && this.config.retries! > 0) {
          // Retry on server errors
          return this.retryRequest(error.config);
        }
        throw new NetworkError(
          error.message,
          error.response?.status,
          error.response?.data
        );
      }
    );
  }

  private async retryRequest(config: AxiosRequestConfig, retries = this.config.retries!): Promise<any> {
    if (retries <= 0) {
      throw new NetworkError('Max retries exceeded');
    }

    await new Promise(resolve => setTimeout(resolve, 1000 * (this.config.retries! - retries + 1)));
    return this.httpClient.request(config).catch(() => this.retryRequest(config, retries - 1));
  }

  // Vote management
  async createVote(config: VoteConfig): Promise<string> {
    try {
      const response = await this.httpClient.post('/api/v1/votes', config);
      return response.data.vote_id;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  async getVote(voteId: string): Promise<Vote> {
    try {
      const response = await this.httpClient.get(`/api/v1/votes/${voteId}`);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  async listVotes(query: ListQuery): Promise<Page<Vote>> {
    try {
      const response = await this.httpClient.get('/api/v1/votes', { params: query });
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  async getVoteStats(voteId: string): Promise<VoteStats> {
    try {
      const response = await this.httpClient.get(`/api/v1/votes/${voteId}/stats`);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  async getVotePhase(voteId: string): Promise<VotePhase> {
    try {
      const response = await this.httpClient.get(`/api/v1/votes/${voteId}/phase`);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // Commitment operations
  async commitVote(voteId: string, request: CommitRequest): Promise<CommitResponse> {
    try {
      const response = await this.httpClient.post(`/api/v1/votes/${voteId}/commit`, request);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // Reveal operations
  async revealVote(voteId: string, request: RevealRequest): Promise<RevealResponse> {
    try {
      const response = await this.httpClient.post(`/api/v1/votes/${voteId}/reveal`, request);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // Results
  async getResults(voteId: string): Promise<VoteResults> {
    try {
      const response = await this.httpClient.get(`/api/v1/votes/${voteId}/results`);
      return response.data;
    } catch (error) {
      throw this.handleError(error);
    }
  }

  // WebSocket connection
  connectWebSocket(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (!this.config.websocketUrl) {
        reject(new VoteError('WebSocket URL not configured'));
        return;
      }

      this.wsClient = new WebSocket(this.config.websocketUrl);

      this.wsClient.on('open', () => {
        console.log('WebSocket connected');
        resolve();
      });

      this.wsClient.on('message', (data: WebSocket.Data) => {
        try {
          const event: WebSocketEvent = JSON.parse(data.toString());
          this.handleWebSocketEvent(event);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      });

      this.wsClient.on('error', (error) => {
        console.error('WebSocket error:', error);
        reject(new VoteError(`WebSocket error: ${error.message}`));
      });

      this.wsClient.on('close', () => {
        console.log('WebSocket disconnected');
      });
    });
  }

  disconnectWebSocket(): void {
    if (this.wsClient) {
      this.wsClient.close();
      this.wsClient = undefined;
    }
  }

  // Event handling
  on(eventType: string, handler: (event: WebSocketEvent) => void): void {
    if (!this.eventHandlers.has(eventType)) {
      this.eventHandlers.set(eventType, []);
    }
    this.eventHandlers.get(eventType)!.push(handler);
  }

  off(eventType: string, handler: (event: WebSocketEvent) => void): void {
    const handlers = this.eventHandlers.get(eventType);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
      }
    }
  }

  private handleWebSocketEvent(event: WebSocketEvent): void {
    const handlers = this.eventHandlers.get(event.type);
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(event);
        } catch (error) {
          console.error(`Error in event handler for ${event.type}:`, error);
        }
      });
    }
  }

  // Utility methods
  private handleError(error: any): VoteError {
    if (error instanceof VoteError || error instanceof NetworkError) {
      return error;
    }

    if (error.response) {
      return new VoteError(
        error.response.data?.message || 'API request failed',
        error.response.data?.code,
        error.response.data
      );
    }

    return new VoteError(error.message || 'Unknown error occurred');
  }

  // Health check
  async healthCheck(): Promise<boolean> {
    try {
      await this.httpClient.get('/health');
      return true;
    } catch (error) {
      return false;
    }
  }
}

// Factory function
export function createVoteClient(config: VoteSDKConfig): VoteClient {
  return new VoteClient(config);
}

// Default export
export default VoteClient;
