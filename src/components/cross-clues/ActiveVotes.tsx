'use client'

import { useState } from 'react'
import { Button } from '@/components/Button'
import { CrossCluesBoard } from './CrossCluesBoard'
import type { CrossCluesState, CrossCluesVote, Player } from '@/lib/types'

interface ActiveVotesProps {
  crossClues: CrossCluesState
  players: Player[]
  playerId: string
  onVote: (voteId: string, coordinate: string) => Promise<void>
  onForceResolve?: (voteId: string) => Promise<void>
}

function VoteCard({
  vote,
  crossClues,
  players,
  playerId,
  onVote,
  onForceResolve,
}: {
  vote: CrossCluesVote
  crossClues: CrossCluesState
  players: Player[]
  playerId: string
  onVote: (voteId: string, coordinate: string) => Promise<void>
  onForceResolve?: (voteId: string) => Promise<void>
}) {
  const [selectedCoord, setSelectedCoord] = useState<string | null>(null)
  const [isVoting, setIsVoting] = useState(false)

  const cluer = players.find(p => p.id === vote.cluerId)
  const isCluer = vote.cluerId === playerId
  const hasVoted = vote.votes[playerId] !== undefined
  const voteCount = Object.keys(vote.votes).length
  const eligibleVoters = players.filter(p => p.id !== vote.cluerId).length

  const handleVote = async () => {
    if (!selectedCoord) return

    setIsVoting(true)
    try {
      await onVote(vote.id, selectedCoord)
    } finally {
      setIsVoting(false)
    }
  }

  const handleForceResolve = async () => {
    if (onForceResolve) {
      await onForceResolve(vote.id)
    }
  }

  return (
    <div className="p-4 bg-blue-50 rounded-lg border-2 border-blue-300 mb-3">
      <div className="flex justify-between items-start mb-3">
        <div>
          <p className="font-secondary text-sm text-gray-600">
            {cluer?.username || 'Unknown'} says:
          </p>
          <p className="font-primary text-2xl text-blue-700">&ldquo;{vote.clue}&rdquo;</p>
        </div>
        <div className="text-right">
          <p className="font-secondary text-xs text-gray-500">
            {voteCount}/{eligibleVoters} voted
          </p>
        </div>
      </div>

      {isCluer ? (
        <div className="text-center py-2">
          <p className="font-secondary text-sm text-gray-500">
            Waiting for others to vote on your clue...
          </p>
          {voteCount > 0 && onForceResolve && (
            <Button
              variant="secondary"
              size="small"
              onClick={handleForceResolve}
              className="mt-2"
            >
              Force Resolve ({voteCount} votes)
            </Button>
          )}
        </div>
      ) : hasVoted ? (
        <div className="text-center py-2">
          <p className="font-secondary text-sm text-green-600">
            You voted: <strong>{vote.votes[playerId]}</strong>
          </p>
          <p className="font-secondary text-xs text-gray-500 mt-1">
            Waiting for others...
          </p>
        </div>
      ) : (
        <div>
          <p className="font-secondary text-sm text-gray-600 mb-2">
            Select a cell:
          </p>
          <div className="mb-3">
            <CrossCluesBoard
              crossClues={crossClues}
              onCellClick={(coord) => setSelectedCoord(coord)}
              selectedCell={selectedCoord}
            />
          </div>
          <Button
            variant="primary"
            size="small"
            onClick={handleVote}
            disabled={!selectedCoord || isVoting}
            className="w-full"
          >
            {isVoting ? 'Voting...' : 'Submit Vote'}
          </Button>
        </div>
      )}
    </div>
  )
}

export function ActiveVotes({
  crossClues,
  players,
  playerId,
  onVote,
  onForceResolve,
}: ActiveVotesProps) {
  const { activeVotes } = crossClues

  if (activeVotes.length === 0) {
    return (
      <div className="text-center py-4">
        <p className="font-secondary text-gray-500">No active votes</p>
        <p className="font-secondary text-sm text-gray-400 mt-1">
          Give a clue to start a vote!
        </p>
      </div>
    )
  }

  return (
    <div>
      {activeVotes.map(vote => (
        <VoteCard
          key={vote.id}
          vote={vote}
          crossClues={crossClues}
          players={players}
          playerId={playerId}
          onVote={onVote}
          onForceResolve={onForceResolve}
        />
      ))}
    </div>
  )
}
