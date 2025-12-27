import { NextRequest, NextResponse } from 'next/server'
import { joinGame } from '@/lib/game'

// POST /api/games/join - Join an existing game
export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { code, username } = body

    if (!username || typeof username !== 'string' || username.trim().length === 0) {
      return NextResponse.json({ error: 'Username is required' }, { status: 400 })
    }

    if (username.length > 20) {
      return NextResponse.json({ error: 'Username must be 20 characters or less' }, { status: 400 })
    }

    if (!code || typeof code !== 'string' || code.trim().length !== 6) {
      return NextResponse.json({ error: 'Valid 6-character game code is required' }, { status: 400 })
    }

    const result = await joinGame(code.trim().toUpperCase(), username.trim())

    if (!result) {
      return NextResponse.json({ error: 'Game not found or username already taken' }, { status: 404 })
    }

    return NextResponse.json({
      game: result.game,
      playerId: result.playerId,
    })
  } catch (error) {
    console.error('Error joining game:', error)
    return NextResponse.json({ error: 'Failed to join game' }, { status: 500 })
  }
}
