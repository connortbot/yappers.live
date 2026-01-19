import { NextRequest, NextResponse } from 'next/server'
import { castVote } from '@/lib/game'
import { validatePlayerId, validateVoteId, validateCoordinate } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: gameId } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)
    const voteId = validateVoteId(body.voteId)
    const coordinate = validateCoordinate(body.coordinate)

    const game = await castVote(gameId, playerId, voteId, coordinate)

    if (!game) {
      return NextResponse.json(
        { error: 'Cannot cast vote. Vote may not exist or you may be the cluer.' },
        { status: 400 }
      )
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error casting vote:', error)
    return NextResponse.json({ error: 'Failed to cast vote' }, { status: 500 })
  }
}
