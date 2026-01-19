import { NextRequest, NextResponse } from 'next/server'
import { startCrossClues } from '@/lib/game'
import { validatePlayerId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: gameId } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)

    const game = await startCrossClues(gameId, playerId)

    if (!game) {
      return NextResponse.json(
        { error: 'Cannot start game. Make sure you are the host and have enough players.' },
        { status: 400 }
      )
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error starting Cross Clues:', error)
    return NextResponse.json({ error: 'Failed to start game' }, { status: 500 })
  }
}
