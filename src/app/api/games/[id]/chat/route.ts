import { NextRequest, NextResponse } from 'next/server'
import { sendChatMessage } from '@/lib/game'
import { validatePlayerId, validateMessage } from '@/lib/validation'

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params
    const body = await request.json()
    const playerId = validatePlayerId(body.playerId)
    const message = validateMessage(body.message)

    const game = await sendChatMessage(id, playerId, message)

    if (!game) {
      return NextResponse.json({ error: 'Game not found or player not in game' }, { status: 404 })
    }

    return NextResponse.json({ game })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error sending chat message:', error)
    return NextResponse.json({ error: 'Failed to send message' }, { status: 500 })
  }
}
