import { NextRequest, NextResponse } from 'next/server'
import { joinGame } from '@/lib/game'
import { validateUsername, validateGameCode } from '@/lib/validation'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const username = validateUsername(body.username)
    const code = validateGameCode(body.code)

    const result = await joinGame(code, username)

    if (!result) {
      return NextResponse.json({ error: 'Game not found or username already taken' }, { status: 404 })
    }

    return NextResponse.json({
      game: result.game,
      playerId: result.playerId,
    })
  } catch (error) {
    if (error instanceof Error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }
    console.error('Error joining game:', error)
    return NextResponse.json({ error: 'Failed to join game' }, { status: 500 })
  }
}
