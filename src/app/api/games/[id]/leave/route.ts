import { NextRequest, NextResponse } from 'next/server'
import { leaveGame } from '@/lib/game'
import { validatePlayerId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)

    const game = await leaveGame(id, playerId)

    return NextResponse.json({ game, deleted: game === null })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error leaving game:', error)
    return NextResponse.json({ error: 'Failed to leave game' }, { status: 500 })
  }
}
