import { NextRequest, NextResponse } from 'next/server'
import { endCrossClues } from '@/lib/game'
import { validatePlayerId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: gameId } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)

    const game = await endCrossClues(gameId, playerId)

    if (!game) {
      return NextResponse.json(
        { error: 'Cannot end game. Make sure you are the host.' },
        { status: 400 }
      )
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error ending Cross Clues:', error)
    return NextResponse.json({ error: 'Failed to end game' }, { status: 500 })
  }
}
