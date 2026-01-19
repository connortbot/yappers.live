import { redis } from './redis'
import { v4 as uuidv4 } from 'uuid'
import type { Game, Player, ChatMessage, GameMode, CrossCluesState, CrossCluesVote } from './types'
import { getRandomWords, generateCards } from './cross-clues-words'

const GAME_EXPIRY_HOURS = 24
const GAME_EXPIRY_SECONDS = GAME_EXPIRY_HOURS * 60 * 60
const MAX_CHAT_MESSAGES = 100
const CODE_CHARS = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'
const CODE_LENGTH = 6
const MIN_PLAYERS_FOR_ROUND = 3
const MIN_PLAYERS_FOR_CROSS_CLUES = 2
const PLAYER_TIMEOUT_MS = 30000 // 30 seconds for presence tracking

function generateCode(): string {
  let code = ''
  for (let i = 0; i < CODE_LENGTH; i++) {
    code += CODE_CHARS.charAt(Math.floor(Math.random() * CODE_CHARS.length))
  }
  return code
}

/**
 * Checks if a player is considered online based on their lastSeenAt timestamp.
 */
export function isPlayerOnline(player: Player): boolean {
  return Date.now() - player.lastSeenAt < PLAYER_TIMEOUT_MS
}

/**
 * Returns only the players who are considered online.
 */
export function getOnlinePlayers(game: Game): Player[] {
  return game.players.filter(isPlayerOnline)
}

/**
 * Updates the lastSeenAt timestamp for a player.
 */
function updatePlayerActivity(game: Game, playerId: string): void {
  const player = game.players.find(p => p.id === playerId)
  if (player) {
    player.lastSeenAt = Date.now()
  }
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
 * @param gameMode - The game mode to play (defaults to 'yappers')
 * @returns The created game and the host's player ID
 */
export async function createGame(username: string, gameMode: GameMode = 'yappers'): Promise<{ game: Game; playerId: string }> {
  const client = redis()
  const gameId = uuidv4()
  const playerId = uuidv4()

  let code = generateCode()
  while (await client.exists(`code:${code}`)) {
    code = generateCode()
  }

  const now = Date.now()
  const player: Player = { id: playerId, username, lastSeenAt: now }

  const game: Game = {
    id: gameId,
    code,
    hostId: playerId,
    players: [player],
    state: 'lobby',
    gameMode,
    round: null,
    roundHistory: [],
    thingPool: [],
    crossClues: null,
    chat: [],
    createdAt: now,
    lastActivityAt: now,
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
  const player: Player = { id: playerId, username, lastSeenAt: Date.now() }

  game.players.push(player)
  await saveGame(game)

  return { game, playerId }
}

/**
 * Verifies a player is in a game (for reconnection).
 * Updates the player's lastSeenAt timestamp to mark them as active.
 * @param gameId - The game's UUID
 * @param playerId - The player's UUID
 * @returns The game if player is a member, null otherwise
 */
export async function rejoinGame(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null

  const isInGame = game.players.some(p => p.id === playerId)
  if (!isInGame) return null

  // Update player presence
  updatePlayerActivity(game, playerId)
  await saveGame(game)

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

// ============================================
// CROSS CLUES GAME LOGIC
// ============================================

/**
 * Initializes and starts a Cross Clues game.
 * @param gameId - The game's UUID
 * @param playerId - The requesting player's ID (must be host)
 * @returns The updated game, or null if conditions not met
 */
export async function startCrossClues(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null

  if (game.hostId !== playerId) return null
  if (game.gameMode !== 'cross-clues') return null
  if (game.players.length < MIN_PLAYERS_FOR_CROSS_CLUES) return null
  if (game.state === 'playing') return null

  updatePlayerActivity(game, playerId)

  // Initialize Cross Clues state
  const { rowWords, colWords } = getRandomWords()
  const cards = generateCards()

  const crossCluesState: CrossCluesState = {
    grid: {},
    rowWords,
    colWords,
    cards,
    activeVotes: [],
    completedVotes: [],
    score: 0
  }

  game.crossClues = crossCluesState
  game.state = 'playing'

  // Assign initial cards to all players
  for (const player of game.players) {
    assignCardToPlayer(game, player.id)
  }

  await saveGame(game)
  return game
}

/**
 * Assigns the next available card to a player.
 * Internal helper function.
 */
function assignCardToPlayer(game: Game, playerId: string): boolean {
  if (!game.crossClues) return false

  // Check if player already has a card
  const existingCard = game.crossClues.cards.find(c => c.assignedTo === playerId)
  if (existingCard) return false

  // Find next unassigned card
  const availableCard = game.crossClues.cards.find(c => c.assignedTo === null)
  if (!availableCard) return false

  availableCard.assignedTo = playerId
  return true
}

/**
 * Gets the card currently assigned to a player.
 */
export function getPlayerCard(game: Game, playerId: string): { coordinate: string, cardId: string } | null {
  if (!game.crossClues) return null

  const card = game.crossClues.cards.find(c => c.assignedTo === playerId)
  if (!card) return null

  return { coordinate: card.coordinate, cardId: card.id }
}

/**
 * Submits a clue for a card and starts a vote.
 * @param gameId - The game's UUID
 * @param playerId - The player giving the clue
 * @param clue - The one-word clue
 * @returns The updated game, or null if conditions not met
 */
export async function submitClue(gameId: string, playerId: string, clue: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  if (!game.crossClues) return null
  if (game.state !== 'playing') return null

  updatePlayerActivity(game, playerId)

  // Find the player's card
  const card = game.crossClues.cards.find(c => c.assignedTo === playerId)
  if (!card) return null

  // Check if there's already an active vote for this card
  const existingVote = game.crossClues.activeVotes.find(v => v.cardId === card.id)
  if (existingVote) return null

  // Create the vote
  const vote: CrossCluesVote = {
    id: uuidv4(),
    cardId: card.id,
    coordinate: card.coordinate,
    clue: clue.trim(),
    cluerId: playerId,
    votes: {},
    status: 'active',
    result: null,
    createdAt: Date.now()
  }

  game.crossClues.activeVotes.push(vote)

  await saveGame(game)
  return game
}

/**
 * Casts a vote on an active vote.
 * @param gameId - The game's UUID
 * @param playerId - The voting player's ID
 * @param voteId - The vote ID
 * @param coordinate - The guessed coordinate (e.g., "A3")
 * @returns The updated game, or null if conditions not met
 */
export async function castVote(gameId: string, playerId: string, voteId: string, coordinate: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  if (!game.crossClues) return null
  if (game.state !== 'playing') return null

  updatePlayerActivity(game, playerId)

  // Find the vote
  const vote = game.crossClues.activeVotes.find(v => v.id === voteId)
  if (!vote) return null
  if (vote.status !== 'active') return null

  // Cluer cannot vote on their own clue
  if (vote.cluerId === playerId) return null

  // Validate coordinate format
  if (!/^[A-E][1-5]$/.test(coordinate)) return null

  // Record the vote
  vote.votes[playerId] = coordinate

  // Check if all online players (except cluer) have voted
  const onlinePlayers = getOnlinePlayers(game)
  const eligibleVoters = onlinePlayers.filter(p => p.id !== vote.cluerId)
  const allVoted = eligibleVoters.every(p => vote.votes[p.id] !== undefined)

  if (allVoted && eligibleVoters.length > 0) {
    // Resolve the vote
    resolveVote(game, vote)
  }

  await saveGame(game)
  return game
}

/**
 * Resolves a vote and updates the grid accordingly.
 * Internal helper function.
 */
function resolveVote(game: Game, vote: CrossCluesVote): void {
  if (!game.crossClues) return

  vote.status = 'resolved'

  // Check if all votes are correct
  const allCorrect = Object.values(vote.votes).every(v => v === vote.coordinate)

  if (allCorrect && Object.keys(vote.votes).length > 0) {
    // Success - fill the square
    vote.result = 'success'
    game.crossClues.grid[vote.coordinate] = 'filled'
    game.crossClues.score++
  } else {
    // Failure - discard the card
    vote.result = 'failure'
    game.crossClues.grid[vote.coordinate] = 'discarded'
  }

  // Move vote to completed
  game.crossClues.activeVotes = game.crossClues.activeVotes.filter(v => v.id !== vote.id)
  game.crossClues.completedVotes.push(vote)

  // Remove the card from the cluer
  const card = game.crossClues.cards.find(c => c.id === vote.cardId)
  if (card) {
    card.assignedTo = null
  }

  // Assign a new card to the cluer
  assignCardToPlayer(game, vote.cluerId)

  // Check if game is over (all squares filled or discarded)
  const totalSquares = 25
  const resolvedSquares = Object.keys(game.crossClues.grid).length
  if (resolvedSquares >= totalSquares) {
    game.state = 'lobby'
  }
}

/**
 * Ends a Cross Clues game early.
 * @param gameId - The game's UUID
 * @param playerId - The requesting player's ID (must be host)
 * @returns The updated game, or null if conditions not met
 */
export async function endCrossClues(gameId: string, playerId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null

  if (game.hostId !== playerId) return null
  if (game.gameMode !== 'cross-clues') return null
  if (game.state !== 'playing') return null

  game.state = 'lobby'

  await saveGame(game)
  return game
}

/**
 * Force-resolves a vote if it's been stuck (e.g., when players go offline).
 * Can be called by anyone if at least one vote has been cast.
 * @param gameId - The game's UUID
 * @param playerId - The requesting player's ID
 * @param voteId - The vote to force-resolve
 * @returns The updated game, or null if conditions not met
 */
export async function forceResolveVote(gameId: string, playerId: string, voteId: string): Promise<Game | null> {
  const game = await getGame(gameId)
  if (!game) return null
  if (!game.crossClues) return null

  updatePlayerActivity(game, playerId)

  const vote = game.crossClues.activeVotes.find(v => v.id === voteId)
  if (!vote) return null
  if (vote.status !== 'active') return null

  // Must have at least one vote to force resolve
  if (Object.keys(vote.votes).length === 0) return null

  resolveVote(game, vote)

  await saveGame(game)
  return game
}
