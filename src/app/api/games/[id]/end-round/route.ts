import { NextRequest, NextResponse } from 'next/server'
import { endRound } from '@/lib/game'

// POST /api/games/[id]/end-round - End the current round
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

    const game = await endRound(id, playerId)

    if (!game) {
      return NextResponse.json({ 
        error: 'Cannot end round. Make sure you are the host and a round is in progress.' 
      }, { status: 400 })
    }

    return NextResponse.json({ game })
  } catch (error) {
    console.error('Error ending round:', error)
    return NextResponse.json({ error: 'Failed to end round' }, { status: 500 })
  }
}
