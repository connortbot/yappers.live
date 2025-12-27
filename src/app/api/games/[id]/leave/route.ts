import { NextRequest, NextResponse } from 'next/server'
import { leaveGame } from '@/lib/game'

// POST /api/games/[id]/leave - Leave the game
export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const body = await request.json()
    const { playerId } = body

    if (!playerId || typeof playerId !== 'string') {
      return NextResponse.json({ error: 'Player ID is required' }, { status: 400 })
    }

    const game = await leaveGame(id, playerId)

    // game will be null if the player was the last one (game deleted)
    return NextResponse.json({ game, deleted: game === null })
  } catch (error) {
    console.error('Error leaving game:', error)
    return NextResponse.json({ error: 'Failed to leave game' }, { status: 500 })
  }
}
