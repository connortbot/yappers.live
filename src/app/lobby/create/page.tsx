'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/Button'
import { Input } from '@/components/Input'
import { Screen } from '@/components/Screen'
import { Section } from '@/components/Section'
import { AnimatedBackground } from '@/components/AnimatedBackground'
import { useLocalPlayer } from '@/hooks/useLocalPlayer'
import type { GameMode } from '@/lib/types'

export default function CreateGame() {
  const router = useRouter()
  const { savePlayer } = useLocalPlayer()
  const [username, setUsername] = useState('')
  const [gameMode, setGameMode] = useState<GameMode>('yappers')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleCreate = async () => {
    if (!username.trim()) {
      setError('Please enter a username')
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      const response = await fetch('/api/games', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: username.trim(), gameMode }),
      })

      const data = await response.json()

      if (!response.ok) {
        throw new Error(data.error || 'Failed to create game')
      }

      // Save player info to localStorage
      savePlayer(data.playerId, data.game.id, username.trim())

      // Navigate to game
      router.push(`/game/${data.game.id}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create game')
    } finally {
      setIsLoading(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleCreate()
    }
  }

  return (
    <>
      <AnimatedBackground />
      <Screen centered>
        <Section title="Create Game" className="w-full max-w-sm">
          {error && (
            <div className="mb-4 p-3 bg-red-100 border-2 border-red-500 rounded-lg">
              <p className="text-red-700 font-secondary text-sm">{error}</p>
            </div>
          )}
          
          <div className="space-y-4">
            <div>
              <label className="block font-secondary text-pencil text-sm mb-1">
                Your Name
              </label>
              <Input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Enter your name"
                maxLength={20}
                disabled={isLoading}
              />
            </div>

            <div>
              <label className="block font-secondary text-pencil text-sm mb-2">
                Game Mode
              </label>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setGameMode('yappers')}
                  disabled={isLoading}
                  className={`flex-1 p-3 rounded-lg border-2 transition-all font-secondary text-sm ${
                    gameMode === 'yappers'
                      ? 'border-black bg-amber-100 text-black'
                      : 'border-gray-300 bg-white text-pencil hover:border-gray-400'
                  }`}
                >
                  <div className="font-bold">Yappers</div>
                  <div className="text-xs mt-1 opacity-75">Find the spy!</div>
                </button>
                <button
                  type="button"
                  onClick={() => setGameMode('cross-clues')}
                  disabled={isLoading}
                  className={`flex-1 p-3 rounded-lg border-2 transition-all font-secondary text-sm ${
                    gameMode === 'cross-clues'
                      ? 'border-black bg-amber-100 text-black'
                      : 'border-gray-300 bg-white text-pencil hover:border-gray-400'
                  }`}
                >
                  <div className="font-bold">Cross Clues</div>
                  <div className="text-xs mt-1 opacity-75">Word grid co-op</div>
                </button>
              </div>
            </div>

            <div className="flex gap-3">
              <Button
                variant="secondary"
                size="medium"
                onClick={() => router.push('/')}
                disabled={isLoading}
                className="flex-1"
              >
                Back
              </Button>
              <Button
                variant="primary"
                size="medium"
                onClick={handleCreate}
                disabled={isLoading || !username.trim()}
                className="flex-1"
              >
                {isLoading ? 'Creating...' : 'Create'}
              </Button>
            </div>
          </div>
        </Section>
      </Screen>
    </>
  )
}
