export interface Player {
  id: string
  username: string
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
  round: Round | null
  roundHistory: Round[]     // past rounds
  thingPool: string[]       // defaults to player names
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
