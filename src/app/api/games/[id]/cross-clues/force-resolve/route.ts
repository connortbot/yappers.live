import { NextRequest, NextResponse } from 'next/server'
import { forceResolveVote } from '@/lib/game'
import { validatePlayerId, validateVoteId } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id: gameId } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)
    const voteId = validateVoteId(body.voteId)

    const game = await forceResolveVote(gameId, playerId, voteId)

    if (!game) {
      return NextResponse.json(
        { error: 'Cannot force resolve. Vote must have at least one vote.' },
        { status: 400 }
      )
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error force resolving vote:', error)
    return NextResponse.json({ error: 'Failed to force resolve vote' }, { status: 500 })
  }
}
