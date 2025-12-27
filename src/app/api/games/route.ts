import { NextRequest, NextResponse } from 'next/server'
import { createGame } from '@/lib/game'

// POST /api/games - Create a new game
export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { username } = body

    if (!username || typeof username !== 'string' || username.trim().length === 0) {
      return NextResponse.json({ error: 'Username is required' }, { status: 400 })
    }

    if (username.length > 20) {
      return NextResponse.json({ error: 'Username must be 20 characters or less' }, { status: 400 })
    }

    const result = await createGame(username.trim())

    return NextResponse.json({
      game: result.game,
      playerId: result.playerId,
    })
  } catch (error) {
    console.error('Error creating game:', error)
    return NextResponse.json({ error: 'Failed to create game' }, { status: 500 })
  }
}
