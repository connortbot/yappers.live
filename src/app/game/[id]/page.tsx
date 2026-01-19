'use client'

import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'
import { Button } from '@/components/Button'
import { Screen } from '@/components/Screen'
import { Section } from '@/components/Section'
import { AnimatedBackground } from '@/components/AnimatedBackground'
import { GameCodeDisplay } from '@/components/GameCodeDisplay'
import { PlayerList } from '@/components/PlayerList'
import { ChatBox } from '@/components/ChatBox'
import { CrossCluesGame } from '@/components/cross-clues/CrossCluesGame'
import { useLocalPlayer } from '@/hooks/useLocalPlayer'
import { useGame } from '@/hooks/useGame'

export default function GamePage() {
  const params = useParams()
  const router = useRouter()
  const gameId = params.id as string
  const { playerId, username, isLoaded, clearPlayer } = useLocalPlayer()
  const { game, isLoading, error } = useGame({
    gameId,
    playerId,
    enabled: isLoaded && !!playerId
  })

  const [actionLoading, setActionLoading] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)

  // Redirect to home if no player data
  useEffect(() => {
    if (isLoaded && !playerId) {
      router.push('/')
    }
  }, [isLoaded, playerId, router])

  // Handle leaving the game
  const handleLeave = async () => {
    if (!playerId) return

    setActionLoading(true)
    try {
      await fetch(`/api/games/${gameId}/leave`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId }),
      })
      clearPlayer()
      router.push('/')
    } catch (e) {
      console.error('Failed to leave game:', e)
      clearPlayer()
      router.push('/')
    } finally {
      setActionLoading(false)
    }
  }

  // Handle sending a chat message
  const handleSendMessage = async (message: string) => {
    if (!playerId) return

    try {
      await fetch(`/api/games/${gameId}/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId, message }),
      })
    } catch (e) {
      console.error('Failed to send message:', e)
    }
  }

  // Handle starting a round (Yappers)
  const handleStartRound = async () => {
    if (!playerId) return

    setActionLoading(true)
    setActionError(null)

    try {
      const response = await fetch(`/api/games/${gameId}/start-round`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId }),
      })

      if (!response.ok) {
        const data = await response.json()
        throw new Error(data.error || 'Failed to start round')
      }
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to start round')
    } finally {
      setActionLoading(false)
    }
  }

  // Handle ending a round (Yappers)
  const handleEndRound = async () => {
    if (!playerId) return

    setActionLoading(true)
    setActionError(null)

    try {
      const response = await fetch(`/api/games/${gameId}/end-round`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId }),
      })

      if (!response.ok) {
        const data = await response.json()
        throw new Error(data.error || 'Failed to end round')
      }
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to end round')
    } finally {
      setActionLoading(false)
    }
  }

  // Cross Clues handlers
  const handleStartCrossClues = async () => {
    if (!playerId) return

    setActionLoading(true)
    setActionError(null)

    try {
      const response = await fetch(`/api/games/${gameId}/cross-clues/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId }),
      })

      if (!response.ok) {
        const data = await response.json()
        throw new Error(data.error || 'Failed to start game')
      }
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to start game')
    } finally {
      setActionLoading(false)
    }
  }

  const handleEndCrossClues = async () => {
    if (!playerId) return

    setActionLoading(true)
    setActionError(null)

    try {
      const response = await fetch(`/api/games/${gameId}/cross-clues/end`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ playerId }),
      })

      if (!response.ok) {
        const data = await response.json()
        throw new Error(data.error || 'Failed to end game')
      }
    } catch (e) {
      setActionError(e instanceof Error ? e.message : 'Failed to end game')
    } finally {
      setActionLoading(false)
    }
  }

  const handleSubmitClue = async (clue: string) => {
    if (!playerId) return

    const response = await fetch(`/api/games/${gameId}/cross-clues/clue`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ playerId, clue }),
    })

    if (!response.ok) {
      const data = await response.json()
      throw new Error(data.error || 'Failed to submit clue')
    }
  }

  const handleVote = async (voteId: string, coordinate: string) => {
    if (!playerId) return

    const response = await fetch(`/api/games/${gameId}/cross-clues/vote`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ playerId, voteId, coordinate }),
    })

    if (!response.ok) {
      const data = await response.json()
      throw new Error(data.error || 'Failed to cast vote')
    }
  }

  const handleForceResolve = async (voteId: string) => {
    if (!playerId) return

    const response = await fetch(`/api/games/${gameId}/cross-clues/force-resolve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ playerId, voteId }),
    })

    if (!response.ok) {
      const data = await response.json()
      throw new Error(data.error || 'Failed to force resolve')
    }
  }

  // Loading state
  if (!isLoaded || isLoading) {
    return (
      <>
        <AnimatedBackground />
        <Screen centered>
          <p className="font-secondary text-pencil">Loading...</p>
        </Screen>
      </>
    )
  }

  // Error state
  if (error || !game) {
    return (
      <>
        <AnimatedBackground />
        <Screen centered>
          <Section variant="error" className="w-full max-w-sm">
            <p className="font-secondary text-red-700 mb-4">
              {error || 'Game not found'}
            </p>
            <Button
              variant="primary"
              size="medium"
              onClick={() => {
                clearPlayer()
                router.push('/')
              }}
            >
              Back to Home
            </Button>
          </Section>
        </Screen>
      </>
    )
  }

  const isHost = game.hostId === playerId
  const isSpy = game.round?.spyId === playerId

  // Cross Clues game mode
  if (game.gameMode === 'cross-clues') {
    return (
      <>
        <AnimatedBackground />
        <Screen>
          <GameCodeDisplay code={game.code} />

          <CrossCluesGame
            game={game}
            playerId={playerId || ''}
            onSendMessage={handleSendMessage}
            onStartGame={handleStartCrossClues}
            onEndGame={handleEndCrossClues}
            onSubmitClue={handleSubmitClue}
            onVote={handleVote}
            onForceResolve={handleForceResolve}
            username={username || ''}
            actionLoading={actionLoading}
            actionError={actionError}
          />

          {/* Leave Game */}
          <div className="mt-4">
            <Button
              variant="secondary"
              size="small"
              onClick={handleLeave}
              disabled={actionLoading}
              className="w-full"
            >
              Leave Game
            </Button>
          </div>
        </Screen>
      </>
    )
  }

  // Yappers game mode (default)
  return (
    <>
      <AnimatedBackground />
      <Screen>
        {/* Game Code */}
        <GameCodeDisplay code={game.code} />

        {/* Action Error */}
        {actionError && (
          <div className="mb-4 p-3 bg-red-100 border-2 border-red-500 rounded-lg">
            <p className="text-red-700 font-secondary text-sm">{actionError}</p>
          </div>
        )}

        {/* Game State Section */}
        {game.state === 'playing' && game.round ? (
          <Section title="Round in Progress">
            <div className="text-center py-6">
              {isSpy ? (
                <>
                  <p className="text-2xl sm:text-3xl font-primary text-red-600 mb-2">
                    You are the SPY!
                  </p>
                  <p className="font-secondary text-pencil">
                    Try to blend in without knowing the thing!
                  </p>
                </>
              ) : (
                <>
                  <p className="font-secondary text-pencil mb-2">
                    The thing is:
                  </p>
                  <p className="text-3xl sm:text-4xl font-primary text-black">
                    {game.round.thing}
                  </p>
                </>
              )}
            </div>

            {isHost && (
              <div className="mt-4">
                <Button
                  variant="secondary"
                  size="medium"
                  onClick={handleEndRound}
                  disabled={actionLoading}
                  className="w-full"
                >
                  {actionLoading ? 'Ending...' : 'End Round'}
                </Button>
              </div>
            )}
          </Section>
        ) : (
          <Section title="Lobby">
            <PlayerList
              players={game.players}
              hostId={game.hostId}
              currentPlayerId={playerId || ''}
            />

            {isHost && (
              <div className="mt-4">
                <Button
                  variant="primary"
                  size="large"
                  onClick={handleStartRound}
                  disabled={actionLoading || game.players.length < 3}
                  className="w-full"
                >
                  {actionLoading ? 'Starting...' : 'Start Round'}
                </Button>
                {game.players.length < 3 && (
                  <p className="text-sm text-pencil font-secondary mt-2 text-center">
                    Need at least 3 players to start
                  </p>
                )}
              </div>
            )}
          </Section>
        )}

        {/* Round History */}
        {game.roundHistory.length > 0 && (
          <Section title="Round History">
            <ul className="space-y-2">
              {game.roundHistory.map((round, index) => {
                const spy = game.players.find(p => p.id === round.spyId)
                return (
                  <li key={index} className="font-secondary text-pencil text-sm">
                    Round {index + 1}: Thing was <strong>{round.thing}</strong>,
                    Spy was <strong>{spy?.username || 'Unknown'}</strong>
                  </li>
                )
              })}
            </ul>
          </Section>
        )}

        {/* Chat */}
        <Section title="Chat">
          <ChatBox
            messages={game.chat}
            onSendMessage={handleSendMessage}
            placeholder={`Chat as ${username}...`}
          />
        </Section>

        {/* Leave Game */}
        <div className="mt-4">
          <Button
            variant="secondary"
            size="small"
            onClick={handleLeave}
            disabled={actionLoading}
            className="w-full"
          >
            Leave Game
          </Button>
        </div>
      </Screen>
    </>
  )
}
