'use client'

import { useState } from 'react'
import { Button } from '@/components/Button'
import { Input } from '@/components/Input'
import type { CrossCluesState } from '@/lib/types'

interface PlayerCardProps {
  crossClues: CrossCluesState
  playerId: string
  onSubmitClue: (clue: string) => Promise<void>
  disabled?: boolean
}

export function PlayerCard({ crossClues, playerId, onSubmitClue, disabled }: PlayerCardProps) {
  const [clue, setClue] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Find the player's card
  const card = crossClues.cards.find(c => c.assignedTo === playerId)

  // Check if player already has an active vote
  const hasActiveVote = crossClues.activeVotes.some(v => v.cluerId === playerId)

  if (!card) {
    return (
      <div className="p-4 bg-gray-100 rounded-lg border-2 border-gray-300 text-center">
        <p className="font-secondary text-pencil">Waiting for a card...</p>
        <p className="font-secondary text-sm text-gray-500 mt-1">
          All cards have been played!
        </p>
      </div>
    )
  }

  // Parse the coordinate to get column and row indices
  const col = card.coordinate.charAt(0)
  const row = card.coordinate.charAt(1)
  const colIndex = col.charCodeAt(0) - 'A'.charCodeAt(0)
  const rowIndex = parseInt(row) - 1

  const colWord = crossClues.colWords[colIndex]
  const rowWord = crossClues.rowWords[rowIndex]

  const handleSubmit = async () => {
    if (!clue.trim()) {
      setError('Please enter a clue')
      return
    }

    if (clue.trim().includes(' ')) {
      setError('Clue must be a single word')
      return
    }

    setIsSubmitting(true)
    setError(null)

    try {
      await onSubmitClue(clue.trim())
      setClue('')
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to submit clue')
    } finally {
      setIsSubmitting(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSubmit()
    }
  }

  if (hasActiveVote) {
    return (
      <div className="p-4 bg-amber-50 rounded-lg border-2 border-amber-400">
        <div className="text-center">
          <p className="font-secondary text-pencil">Your clue is being voted on!</p>
          <p className="font-primary text-2xl mt-2">{card.coordinate}</p>
          <p className="font-secondary text-sm text-gray-500 mt-1">
            Waiting for everyone to vote...
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="p-4 bg-amber-50 rounded-lg border-2 border-amber-400">
      <div className="text-center mb-4">
        <p className="font-secondary text-pencil text-sm">Your Card:</p>
        <p className="font-primary text-4xl text-amber-700">{card.coordinate}</p>
        <div className="mt-2 flex justify-center gap-4 text-sm">
          <span className="font-secondary">
            <strong>{col}:</strong> {colWord}
          </span>
          <span className="font-secondary">
            <strong>{row}:</strong> {rowWord}
          </span>
        </div>
      </div>

      {error && (
        <div className="mb-3 p-2 bg-red-100 border border-red-300 rounded text-red-700 text-sm font-secondary">
          {error}
        </div>
      )}

      <div className="flex gap-2">
        <Input
          type="text"
          value={clue}
          onChange={(e) => setClue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter one-word clue"
          maxLength={50}
          disabled={disabled || isSubmitting}
          className="flex-1"
        />
        <Button
          variant="primary"
          size="small"
          onClick={handleSubmit}
          disabled={disabled || isSubmitting || !clue.trim()}
        >
          {isSubmitting ? '...' : 'Give Clue'}
        </Button>
      </div>
    </div>
  )
}
