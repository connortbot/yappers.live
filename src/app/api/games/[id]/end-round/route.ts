import { NextRequest, NextResponse } from 'next/server'
import { endRound } from '@/lib/game'
import { validatePlayerId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)

    const game = await endRound(id, playerId)

    if (!game) {
      return NextResponse.json({ 
        error: 'Cannot end round. Make sure you are the host and a round is in progress.' 
      }, { status: 400 })
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error ending round:', error)
    return NextResponse.json({ error: 'Failed to end round' }, { status: 500 })
  }
}
