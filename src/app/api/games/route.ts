import { NextRequest, NextResponse } from 'next/server'
import { createGame } from '@/lib/game'
import { validateUsername, validateGameMode } from '@/lib/validation'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const username = validateUsername(body.username)
    const gameMode = validateGameMode(body.gameMode)
    const result = await createGame(username, gameMode)

    return NextResponse.json({
      game: result.game,
      playerId: result.playerId,
    })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error creating game:', error)
    return NextResponse.json({ error: 'Failed to create game' }, { status: 500 })
  }
}
