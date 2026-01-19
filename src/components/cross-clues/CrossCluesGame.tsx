'use client'

import { Button } from '@/components/Button'
import { Section } from '@/components/Section'
import { PlayerList } from '@/components/PlayerList'
import { ChatBox } from '@/components/ChatBox'
import { CrossCluesBoard } from './CrossCluesBoard'
import { PlayerCard } from './PlayerCard'
import { ActiveVotes } from './ActiveVotes'
import { GameSummary } from './GameSummary'
import type { Game } from '@/lib/types'

interface CrossCluesGameProps {
  game: Game
  playerId: string
  onSendMessage: (message: string) => Promise<void>
  onStartGame: () => Promise<void>
  onEndGame: () => Promise<void>
  onSubmitClue: (clue: string) => Promise<void>
  onVote: (voteId: string, coordinate: string) => Promise<void>
  onForceResolve: (voteId: string) => Promise<void>
  username: string
  actionLoading: boolean
  actionError: string | null
}

export function CrossCluesGame({
  game,
  playerId,
  onSendMessage,
  onStartGame,
  onEndGame,
  onSubmitClue,
  onVote,
  onForceResolve,
  username,
  actionLoading,
  actionError,
}: CrossCluesGameProps) {
  const isHost = game.hostId === playerId
  const isPlaying = game.state === 'playing' && game.crossClues

  // Find player's current card for highlighting
  const playerCard = game.crossClues?.cards.find(c => c.assignedTo === playerId)

  if (!isPlaying) {
    // Lobby state
    return (
      <>
        {actionError && (
          <div className="mb-4 p-3 bg-red-100 border-2 border-red-500 rounded-lg">
            <p className="text-red-700 font-secondary text-sm">{actionError}</p>
          </div>
        )}

        <Section title="Cross Clues Lobby">
          <PlayerList
            players={game.players}
            hostId={game.hostId}
            currentPlayerId={playerId}
            showOnlineStatus
          />

          {isHost && (
            <div className="mt-4">
              <Button
                variant="primary"
                size="large"
                onClick={onStartGame}
                disabled={actionLoading || game.players.length < 2}
                className="w-full"
              >
                {actionLoading ? 'Starting...' : 'Start Game'}
              </Button>
              {game.players.length < 2 && (
                <p className="text-sm text-pencil font-secondary mt-2 text-center">
                  Need at least 2 players to start
                </p>
              )}
            </div>
          )}

          {!isHost && (
            <p className="text-sm text-pencil font-secondary mt-4 text-center">
              Waiting for host to start the game...
            </p>
          )}
        </Section>

        {/* Show previous game results if any */}
        {game.crossClues && game.crossClues.completedVotes.length > 0 && (
          <Section title="Last Game Results">
            <GameSummary crossClues={game.crossClues} players={game.players} />
          </Section>
        )}

        <Section title="Chat">
          <ChatBox
            messages={game.chat}
            onSendMessage={onSendMessage}
            placeholder={`Chat as ${username}...`}
          />
        </Section>
      </>
    )
  }

  // Playing state - crossClues is guaranteed to be non-null here due to isPlaying check
  const crossClues = game.crossClues!

  return (
    <>
      {actionError && (
        <div className="mb-4 p-3 bg-red-100 border-2 border-red-500 rounded-lg">
          <p className="text-red-700 font-secondary text-sm">{actionError}</p>
        </div>
      )}

      {/* Score summary */}
      <Section title="Score">
        <GameSummary crossClues={crossClues} players={game.players} />
      </Section>

      {/* The grid */}
      <Section title="Grid">
        <CrossCluesBoard
          crossClues={crossClues}
          highlightedCell={playerCard?.coordinate}
        />
      </Section>

      {/* Player's card */}
      <Section title="Your Card">
        <PlayerCard
          crossClues={crossClues}
          playerId={playerId}
          onSubmitClue={onSubmitClue}
          disabled={actionLoading}
        />
      </Section>

      {/* Active votes */}
      <Section title="Active Votes">
        <ActiveVotes
          crossClues={crossClues}
          players={game.players}
          playerId={playerId}
          onVote={onVote}
          onForceResolve={onForceResolve}
        />
      </Section>

      {/* Players */}
      <Section title="Players">
        <PlayerList
          players={game.players}
          hostId={game.hostId}
          currentPlayerId={playerId}
          showOnlineStatus
        />
      </Section>

      {/* Chat */}
      <Section title="Chat">
        <ChatBox
          messages={game.chat}
          onSendMessage={onSendMessage}
          placeholder={`Chat as ${username}...`}
        />
      </Section>

      {/* End game (host only) */}
      {isHost && (
        <div className="mt-4">
          <Button
            variant="secondary"
            size="small"
            onClick={onEndGame}
            disabled={actionLoading}
            className="w-full"
          >
            End Game Early
          </Button>
        </div>
      )}
    </>
  )
}
