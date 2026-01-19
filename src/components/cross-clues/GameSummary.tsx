'use client'

import type { CrossCluesState, CrossCluesVote, Player } from '@/lib/types'

interface GameSummaryProps {
  crossClues: CrossCluesState
  players: Player[]
}

function CompletedVote({ vote, players }: { vote: CrossCluesVote; players: Player[] }) {
  const cluer = players.find(p => p.id === vote.cluerId)
  const isSuccess = vote.result === 'success'

  return (
    <div className={`p-2 rounded border text-sm ${
      isSuccess ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
    }`}>
      <div className="flex justify-between items-center">
        <span className="font-secondary">
          <strong>{cluer?.username || 'Unknown'}</strong> said &ldquo;{vote.clue}&rdquo;
        </span>
        <span className={`font-bold ${isSuccess ? 'text-green-600' : 'text-red-600'}`}>
          {vote.coordinate} {isSuccess ? '✓' : '✗'}
        </span>
      </div>
    </div>
  )
}

export function GameSummary({ crossClues, players }: GameSummaryProps) {
  const { completedVotes, grid } = crossClues
  const totalSquares = 25
  const filledCount = Object.values(grid).filter(s => s === 'filled').length
  const discardedCount = Object.values(grid).filter(s => s === 'discarded').length
  const remaining = totalSquares - filledCount - discardedCount

  return (
    <div>
      {/* Score display */}
      <div className="flex justify-around text-center mb-4 p-3 bg-gray-50 rounded-lg">
        <div>
          <p className="font-primary text-3xl text-green-600">{filledCount}</p>
          <p className="font-secondary text-xs text-gray-500">Filled</p>
        </div>
        <div>
          <p className="font-primary text-3xl text-red-500">{discardedCount}</p>
          <p className="font-secondary text-xs text-gray-500">Discarded</p>
        </div>
        <div>
          <p className="font-primary text-3xl text-gray-600">{remaining}</p>
          <p className="font-secondary text-xs text-gray-500">Remaining</p>
        </div>
      </div>

      {/* Completed votes history */}
      {completedVotes.length > 0 && (
        <div>
          <p className="font-secondary text-sm text-gray-600 mb-2">Recent:</p>
          <div className="space-y-1 max-h-32 overflow-y-auto">
            {[...completedVotes].reverse().slice(0, 5).map(vote => (
              <CompletedVote key={vote.id} vote={vote} players={players} />
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
