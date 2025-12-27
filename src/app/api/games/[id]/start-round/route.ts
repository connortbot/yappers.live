import { NextRequest, NextResponse } from 'next/server'
import { startRound } from '@/lib/game'
import { validatePlayerId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)

    const game = await startRound(id, playerId)

    if (!game) {
      return NextResponse.json({ 
        error: 'Cannot start round. Make sure you are the host, have at least 3 players, and are not already playing.' 
      }, { status: 400 })
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error starting round:', error)
    return NextResponse.json({ error: 'Failed to start round' }, { status: 500 })
  }
}
