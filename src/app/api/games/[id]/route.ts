import { NextRequest, NextResponse } from 'next/server'
import { getGame, rejoinGame } from '@/lib/game'

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const playerId = request.headers.get('x-player-id')

    if (playerId) {
      const game = await rejoinGame(id, playerId)
      if (!game) {
        return NextResponse.json({ error: 'Game not found or player not in game' }, { status: 404 })
      }
      return NextResponse.json({ game })
    }

    const game = await getGame(id)
    if (!game) {
      return NextResponse.json({ error: 'Game not found' }, { status: 404 })
    }

    return NextResponse.json({ game })
  } catch (error) {
    console.error('Error getting game:', error)
    return NextResponse.json({ error: 'Failed to get game' }, { status: 500 })
  }
}
