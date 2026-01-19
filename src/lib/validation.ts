import type { GameMode } from './types'

export const MAX_USERNAME_LENGTH = 20
export const MAX_MESSAGE_LENGTH = 500
export const GAME_CODE_LENGTH = 6
export const MAX_CLUE_LENGTH = 50
export const VALID_GAME_MODES: GameMode[] = ['yappers', 'cross-clues']

/**
 * Validates and sanitizes a username.
 * @returns Trimmed username if valid
 * @throws Error with user-facing message if invalid
 */
export function validateUsername(value: unknown): string {
  if (!value || typeof value !== 'string' || value.trim().length === 0) {
    throw new Error('Username is required')
  }
  if (value.length > MAX_USERNAME_LENGTH) {
    throw new Error(`Username must be ${MAX_USERNAME_LENGTH} characters or less`)
  }
  return value.trim()
}

/**
 * Validates a player ID.
 * @returns The player ID if valid
 * @throws Error with user-facing message if invalid
 */
export function validatePlayerId(value: unknown): string {
  if (!value || typeof value !== 'string') {
    throw new Error('Player ID is required')
  }
  return value
}

/**
 * Validates and sanitizes a chat message.
 * @returns Trimmed message if valid
 * @throws Error with user-facing message if invalid
 */
export function validateMessage(value: unknown): string {
  if (!value || typeof value !== 'string' || value.trim().length === 0) {
    throw new Error('Message is required')
  }
  if (value.length > MAX_MESSAGE_LENGTH) {
    throw new Error(`Message must be ${MAX_MESSAGE_LENGTH} characters or less`)
  }
  return value.trim()
}

/**
 * Validates and normalizes a game code.
 * @returns Uppercase trimmed code if valid
 * @throws Error with user-facing message if invalid
 */
export function validateGameCode(value: unknown): string {
  if (!value || typeof value !== 'string' || value.trim().length !== GAME_CODE_LENGTH) {
    throw new Error(`Valid ${GAME_CODE_LENGTH}-character game code is required`)
  }
  return value.trim().toUpperCase()
}

/**
 * Validates a game mode.
 * @returns The game mode if valid, defaults to 'yappers'
 */
export function validateGameMode(value: unknown): GameMode {
  if (!value || typeof value !== 'string') {
    return 'yappers'
  }
  if (!VALID_GAME_MODES.includes(value as GameMode)) {
    return 'yappers'
  }
  return value as GameMode
}

/**
 * Validates a Cross Clues clue (one word).
 * @returns Trimmed clue if valid
 * @throws Error with user-facing message if invalid
 */
export function validateClue(value: unknown): string {
  if (!value || typeof value !== 'string' || value.trim().length === 0) {
    throw new Error('Clue is required')
  }
  const trimmed = value.trim()
  if (trimmed.length > MAX_CLUE_LENGTH) {
    throw new Error(`Clue must be ${MAX_CLUE_LENGTH} characters or less`)
  }
  // Check for spaces (should be one word)
  if (trimmed.includes(' ')) {
    throw new Error('Clue must be a single word')
  }
  return trimmed
}

/**
 * Validates a coordinate (e.g., "A3").
 * @returns The coordinate if valid
 * @throws Error with user-facing message if invalid
 */
export function validateCoordinate(value: unknown): string {
  if (!value || typeof value !== 'string') {
    throw new Error('Coordinate is required')
  }
  const coord = value.trim().toUpperCase()
  if (!/^[A-E][1-5]$/.test(coord)) {
    throw new Error('Invalid coordinate format (e.g., A1, B3, E5)')
  }
  return coord
}

/**
 * Validates a vote ID.
 * @returns The vote ID if valid
 * @throws Error with user-facing message if invalid
 */
export function validateVoteId(value: unknown): string {
  if (!value || typeof value !== 'string') {
    throw new Error('Vote ID is required')
  }
  return value
}
