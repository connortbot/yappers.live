'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/Button'
import { Input } from '@/components/Input'
import { Screen } from '@/components/Screen'
import { Section } from '@/components/Section'
import { AnimatedBackground } from '@/components/AnimatedBackground'
import { useLocalPlayer } from '@/hooks/useLocalPlayer'

export default function CreateGame() {
  const router = useRouter()
  const { savePlayer } = useLocalPlayer()
  const [username, setUsername] = useState('')
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
        body: JSON.stringify({ username: username.trim() }),
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
