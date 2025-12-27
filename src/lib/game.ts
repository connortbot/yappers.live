import { redis } from './redis'
import { v4 as uuidv4 } from 'uuid'
import type { Game, Player, ChatMessage } from './types'

const GAME_EXPIRY_HOURS = 24
const GAME_EXPIRY_SECONDS = GAME_EXPIRY_HOURS * 60 * 60
const MAX_CHAT_MESSAGES = 100
const CODE_CHARS = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'
const CODE_LENGTH = 6
const MIN_PLAYERS_FOR_ROUND = 3

function generateCode(): string {
  let code = ''
  for (let i = 0; i < CODE_LENGTH; i++) {
    code += CODE_CHARS.charAt(Math.floor(Math.random() * CODE_CHARS.length))
  }
  return code
}

/**
 * Retrieves a game by its unique ID.
 * @param gameId - The game's UUID
 * @returns The game object or null if not found
 */
export async function getGame(gameId: string): Promise<Game | null> {
  const client = redis()
  const data = await client.get(`game:${gameId}`)
  if (!data) return null
  return JSON.parse(data) as Game
}

/**
 * Retrieves a game by its 6-character join code.
 * @param code - The game's join code (case-insensitive)
 * @returns The game object or null if not found
 */
export async function getGameByCode(code: string): Promise<Game | null> {
  const client = redis()
  const gameId = await client.get(`code:${code.toUpperCase()}`)
  if (!gameId) return null
  return getGame(gameId)
}

async function saveGame(game: Game): Promise<void> {
  const client = redis()
  game.lastActivityAt = Date.now()
  await client.set(`game:${game.id}`, JSON.stringify(game), 'EX', GAME_EXPIRY_SECONDS)
  await client.set(`code:${game.code}`, game.id, 'EX', GAME_EXPIRY_SECONDS)
}

async function deleteGame(game: Game): Promise<void> {
  const client = redis()
  await client.del(`game:${game.id}`)
  await client.del(`code:${game.code}`)
}

/**
 * Creates a new game with the given player as host.
 * @param username - The host player's display name
 * @returns The created game and the host's player ID
 */
export async function createGame(username: string): Promise<{ game: Game; playerId: string }> {
  const client = redis()
  const gameId = uuidv4()
  const playerId = uuidv4()
  
  let code = generateCode()
  while (await client.exists(`code:${code}`)) {
    code = generateCode()
  }
  
  const player: Player = { id: playerId, username }
  
  const game: Game = {
    id: gameId,
    code,
    hostId: playerId,
    players: [player],
    state: 'lobby',
    round: null,
    roundHistory: [],
    thingPool: [],
    chat: [],
    createdAt: Date.now(),
    lastActivityAt: Date.now(),
  }
  
  await saveGame(game)
  return { game, playerId }
}

/**
 * Adds a player to an existing game.
 * @param code - The game's join code
 * @param username - The new player's display name (must be unique in the game)
 * @returns The updated game and new player ID, or null if game not found or username taken
 */
export async function joinGame(code: string, username: string): Promise<{ game: Game; playerId: string } | null> {
  const game = await getGameByCode(code)
  if (!game) return null
  
  const usernameTaken = game.players.some(p => p.username.toLowerCase() === username.toLowerCase())
  if (usernameTaken) return null
  
  const playerId = uuidv4()
  const player: Player = { id: playerId, username }
  
  game.players.push(player)
  await saveGame(game)
  
  return { game, playerId }
}

/**
 * Verifies a player is in a game (for reconnection).
 * @param gameId - The game's UUID
 * @param playerId - The player's UUID
 * @returns The game if player is a member, null otherwise
 */
export async function rejoinGame(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  const isInGame = game.players.some(p => p.id === playerId)
  if (!isInGame) return null
  
  return game
}

/**
 * Removes a player from a game. Handles host reassignment and game cleanup.
 * @param gameId - The game's UUID
 * @param playerId - The leaving player's UUID
 * @returns The updated game, or null if the game was deleted (last player left)
 */
export async function leaveGame(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  game.players = game.players.filter(p => p.id !== playerId)
  
  if (game.players.length === 0) {
    await deleteGame(game)
    return null
  }
  
  if (game.hostId === playerId) {
    game.hostId = game.players[0].id
  }
  
  if (game.state === 'playing' && game.round?.spyId === playerId) {
    game.roundHistory.push(game.round)
    game.round = null
    game.state = 'lobby'
  }
  
  await saveGame(game)
  return game
}

/**
 * Adds a chat message to the game.
 * @param gameId - The game's UUID
 * @param playerId - The sender's player ID
 * @param message - The message content
 * @returns The updated game, or null if game/player not found
 */
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
  
  if (game.chat.length > MAX_CHAT_MESSAGES) {
    game.chat = game.chat.slice(-MAX_CHAT_MESSAGES)
  }
  
  await saveGame(game)
  return game
}

/**
 * Starts a new round. Randomly selects a spy and a "thing" from player names.
 * @param gameId - The game's UUID
 * @param playerId - The requesting player's ID (must be host)
 * @returns The updated game, or null if conditions not met
 */
export async function startRound(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  if (game.hostId !== playerId) return null
  if (game.players.length < MIN_PLAYERS_FOR_ROUND) return null
  if (game.state === 'playing') return null
  
  const spyIndex = Math.floor(Math.random() * game.players.length)
  const spy = game.players[spyIndex]
  
  const thingPool = game.players.map(p => p.username)
  const availableThings = thingPool.filter(t => t !== spy.username)
  const thing = availableThings[Math.floor(Math.random() * availableThings.length)]
  
  game.round = { spyId: spy.id, thing }
  game.state = 'playing'
  game.thingPool = thingPool
  
  await saveGame(game)
  return game
}

/**
 * Ends the current round and saves it to history.
 * @param gameId - The game's UUID
 * @param playerId - The requesting player's ID (must be host)
 * @returns The updated game, or null if conditions not met
 */
export async function endRound(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  
  if (game.hostId !== playerId) return null
  if (game.state !== 'playing' || !game.round) return null
  
  game.roundHistory.push(game.round)
  game.round = null
  game.state = 'lobby'
  
  await saveGame(game)
  return game
}
