export interface Player {
  id: string
  username: string
  lastSeenAt: number  // timestamp for presence tracking
}

export interface ChatMessage {
  username: string
  message: string
  timestamp: number
}

export interface Round {
  spyId: string
  thing: string
}

export interface Game {
  id: string
  code: string              // 6-char join code
  hostId: string
  players: Player[]
  state: 'lobby' | 'playing'
  gameMode: GameMode        // which game mode
  round: Round | null       // yappers-specific
  roundHistory: Round[]     // yappers-specific
  thingPool: string[]       // yappers-specific
  crossClues: CrossCluesState | null  // cross-clues-specific
  chat: ChatMessage[]
  createdAt: number
  lastActivityAt: number    // for expiry
}

// API response types
export interface CreateGameResponse {
  game: Game
  playerId: string
}

export interface JoinGameResponse {
  game: Game
  playerId: string
}

export interface GameResponse {
  game: Game
}

export interface ErrorResponse {
  error: string
}

// Game mode types
export type GameMode = 'yappers' | 'cross-clues'

// Cross Clues specific types
export interface CrossCluesCard {
  id: string              // unique card ID
  coordinate: string      // e.g., "A3"
  assignedTo: string | null  // playerId or null if unassigned
}

export interface CrossCluesVote {
  id: string
  cardId: string          // which card is being voted on
  coordinate: string      // the coordinate being clued
  clue: string            // the one-word clue given
  cluerId: string         // who gave the clue
  votes: Record<string, string>  // playerId -> coordinate guess
  status: 'active' | 'resolved'
  result: 'success' | 'failure' | null
  createdAt: number
}

export interface CrossCluesState {
  grid: Record<string, 'filled' | 'discarded'>  // coordinate -> status (missing = available)
  rowWords: string[]             // 5 words for rows 1-5
  colWords: string[]             // 5 words for columns A-E
  cards: CrossCluesCard[]        // all 25 cards, tracking assignment
  activeVotes: CrossCluesVote[]  // currently active votes
  completedVotes: CrossCluesVote[]  // history of completed votes
  score: number                  // number of filled squares
}
