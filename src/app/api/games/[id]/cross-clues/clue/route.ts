import { NextRequest, NextResponse } from 'next/server'
import { submitClue } from '@/lib/game'
import { validatePlayerId, validateClue } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: gameId } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)
    const clue = validateClue(body.clue)

    const game = await submitClue(gameId, playerId, clue)

    if (!game) {
      return NextResponse.json(
        { error: 'Cannot submit clue. Make sure you have a card and no active vote.' },
        { status: 400 }
      )
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error submitting clue:', error)
    return NextResponse.json({ error: 'Failed to submit clue' }, { status: 500 })
  }
}
