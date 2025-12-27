'use client'

import { useState, useEffect, useCallback } from 'react'
import type { Game } from '@/lib/types'

const POLL_INTERVAL = 2000

interface UseGameOptions {
  gameId: string | null
  playerId: string | null
  enabled?: boolean
}

interface UseGameReturn {
  game: Game | null
  isLoading: boolean
  error: string | null
  refetch: () => Promise<void>
}

export function useGame({ gameId, playerId, enabled = true }: UseGameOptions): UseGameReturn {
  const [game, setGame] = useState<Game | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchGame = useCallback(async () => {
    if (!gameId || !playerId) {
      setIsLoading(false)
      return
    }

    try {
      const response = await fetch(`/api/games/${gameId}`, {
        headers: { 'x-player-id': playerId },
      })

      if (!response.ok) {
        const data = await response.json()
        throw new Error(data.error || 'Failed to fetch game')
      }

      const data = await response.json()
      setGame(data.game)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Unknown error')
    } finally {
      setIsLoading(false)
    }
  }, [gameId, playerId])

  useEffect(() => {
    if (!enabled || !gameId || !playerId) {
      setIsLoading(false)
      return
    }

    fetchGame()

    const interval = setInterval(fetchGame, POLL_INTERVAL)
    return () => clearInterval(interval)
  }, [enabled, gameId, playerId, fetchGame])

  return { game, isLoading, error, refetch: fetchGame }
}
