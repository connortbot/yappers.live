import { redis } from './redis'
import { v4 as uuidv4 } from 'uuid'
import type { Game, Player, ChatMessage } from './types'

const GAME_EXPIRY_HOURS = 24
const GAME_EXPIRY_SECONDS = GAME_EXPIRY_HOURS * 60 * 60

// Generate a random 6-character code
function generateCode(): string {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789' // Avoid confusing chars like 0/O, 1/I
  let code = ''
  for (let i = 0; i < 6; i++) {
    code += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return code
}

// Get game by ID
export async function getGame(gameId: string): Promise<Game | null> {
  const client = redis()
  const data = await client.get(`game:${gameId}`)
  if (!data) return null
  return JSON.parse(data) as Game
}

// Get game by code
export async function getGameByCode(code: string): Promise<Game | null> {
  const client = redis()
  const gameId = await client.get(`code:${code.toUpperCase()}`)
  if (!gameId) return null
  return getGame(gameId)
}

// Save game to Redis
async function saveGame(game: Game): Promise<void> {
  const client = redis()
  game.lastActivityAt = Date.now()
  await client.set(`game:${game.id}`, JSON.stringify(game), 'EX', GAME_EXPIRY_SECONDS)
  await client.set(`code:${game.code}`, game.id, 'EX', GAME_EXPIRY_SECONDS)
}

// Create a new game
export async function createGame(username: string): Promise<{ game: Game; playerId: string }> {
  const client = redis()
  const gameId = uuidv4()
  const playerId = uuidv4()
  
  // Generate unique code
  let code = generateCode()
  while (await client.exists(`code:${code}`)) {
    code = generateCode()
  }
  
  const player: Player = {
    id: playerId,
    username,
  }
  
  const game: Game = {
    id: gameId,
    code,
    hostId: playerId,
    players: [player],
    state: 'lobby',
    round: null,
    roundHistory: [],
    thingPool: [], // Will be populated with player names when round starts
    chat: [],
    createdAt: Date.now(),
    lastActivityAt: Date.now(),
  }
  
  await saveGame(game)
  
  return { game, playerId }
}

// Join an existing game
export async function joinGame(code: string, username: string): Promise<{ game: Game; playerId: string } | null> {
  const game = await getGameByCode(code)
  if (!game) return null
  
  // Check if username already exists in game
  const existingPlayer = game.players.find(p => p.username.toLowerCase() === username.toLowerCase())
  if (existingPlayer) {
    return null // Username taken
  }
  
  const playerId = uuidv4()
  const player: Player = {
    id: playerId,
    username,
  }
  
  game.players.push(player)
  await saveGame(game)
  
  return { game, playerId }
}

// Rejoin a game (for reconnecting players)
export async function rejoinGame(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  // Check if player is in the game
  const player = game.players.find(p => p.id === playerId)
  if (!player) return null
  
  return game
}

// Leave a game
export async function leaveGame(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  // Remove player
  game.players = game.players.filter(p => p.id !== playerId)
  
  // If no players left, delete the game
  if (game.players.length === 0) {
    const client = redis()
    await client.del(`game:${game.id}`)
    await client.del(`code:${game.code}`)
    return null
  }
  
  // If host left, assign new host
  if (game.hostId === playerId) {
    game.hostId = game.players[0].id
  }
  
  // If we were playing and spy left, end the round
  if (game.state === 'playing' && game.round?.spyId === playerId) {
    game.roundHistory.push(game.round)
    game.round = null
    game.state = 'lobby'
  }
  
  await saveGame(game)
  return game
}

// Send a chat message
export async function sendChatMessage(gameId: string, playerId: string, message: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  const player = game.players.find(p => p.id === playerId)
  if (!player) return null
  
  const chatMessage: ChatMessage = {
    username: player.username,
    message,
    timestamp: Date.now(),
  }
  
  game.chat.push(chatMessage)
  
  // Keep only last 100 messages
  if (game.chat.length > 100) {
    game.chat = game.chat.slice(-100)
  }
  
  await saveGame(game)
  return game
}

// Start a new round
export async function startRound(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  // Only host can start round
  if (game.hostId !== playerId) return null
  
  // Need at least 3 players
  if (game.players.length < 3) return null
  
  // Can't start if already playing
  if (game.state === 'playing') return null
  
  // Pick a random spy
  const spyIndex = Math.floor(Math.random() * game.players.length)
  const spy = game.players[spyIndex]
  
  // Build the thing pool from player names (default)
  const thingPool = game.players.map(p => p.username)
  
  // Pick a random thing (excluding the spy's name for fairness)
  const availableThings = thingPool.filter(t => t !== spy.username)
  const thing = availableThings[Math.floor(Math.random() * availableThings.length)]
  
  game.round = {
    spyId: spy.id,
    thing,
  }
  game.state = 'playing'
  game.thingPool = thingPool
  
  await saveGame(game)
  return game
}

// End the current round
export async function endRound(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  // Only host can end round
  if (game.hostId !== playerId) return null
  
  // Must be playing
  if (game.state !== 'playing' || !game.round) return null
  
  // Save to history
  game.roundHistory.push(game.round)
  game.round = null
  game.state = 'lobby'
  
  await saveGame(game)
  return game
}
