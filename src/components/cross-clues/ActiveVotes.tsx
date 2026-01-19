'use client'

import { useState } from 'react'
import { Button } from '@/components/Button'
import type { CrossCluesState, CrossCluesVote, Player } from '@/lib/types'

interface ActiveVotesProps {
  crossClues: CrossCluesState
  players: Player[]
  playerId: string
  onVote: (voteId: string, coordinate: string) => Promise<void>
  onForceResolve?: (voteId: string) => Promise<void>
}

const COLUMNS = ['A', 'B', 'C', 'D', 'E']
const ROWS = ['1', '2', '3', '4', '5']

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

  // Get available cells (not already filled or discarded)
  const getAvailableCells = () => {
    const cells: string[] = []
    for (const col of COLUMNS) {
      for (const row of ROWS) {
        const coord = `${col}${row}`
        if (!crossClues.grid[coord]) {
          cells.push(coord)
        }
      }
    }
    return cells
  }

  const availableCells = getAvailableCells()

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
          <div className="grid grid-cols-5 gap-1 mb-3">
            {availableCells.map(coord => (
              <button
                key={coord}
                type="button"
                onClick={() => setSelectedCoord(coord)}
                className={`p-2 text-sm font-bold rounded border-2 transition-all ${
                  selectedCoord === coord
                    ? 'bg-blue-500 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:border-blue-400'
                }`}
              >
                {coord}
              </button>
            ))}
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
