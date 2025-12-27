export const MAX_USERNAME_LENGTH = 20
export const MAX_MESSAGE_LENGTH = 500
export const GAME_CODE_LENGTH = 6

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
