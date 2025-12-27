'use client'

import { useState, useEffect, useCallback, useRef } from 'react'
import type { Game } from '@/lib/types'

const POLL_INTERVAL = 2000 // 2 seconds

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
  const intervalRef = useRef<NodeJS.Timeout | null>(null)

  const fetchGame = useCallback(async () => {
    if (!gameId || !playerId) {
      setIsLoading(false)
      return
    }

    try {
      const response = await fetch(`/api/games/${gameId}`, {
        headers: {
          'x-player-id': playerId,
        },
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

  // Initial fetch and polling
  useEffect(() => {
    if (!enabled || !gameId || !playerId) {
      setIsLoading(false)
      return
    }

    // Initial fetch
    fetchGame()

    // Set up polling
    intervalRef.current = setInterval(fetchGame, POLL_INTERVAL)

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
      }
    }
  }, [enabled, gameId, playerId, fetchGame])

  return {
    game,
    isLoading,
    error,
    refetch: fetchGame,
  }
}
