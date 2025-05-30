import { useState } from 'react'
import { useParams, useNavigate } from 'react-router'
import { Button } from '../components/Button'
import { Screen } from '../components/Screen'
import { Input } from '../components/Input'
import { Section } from '../components/Section'
import { FormRow } from '../components/FormRow'
import { GameCodeDisplay } from '../components/GameCodeDisplay'
import { PlayerList } from '../components/PlayerList'
import { ChatBox } from '../components/ChatBox'
import { useGameContext } from '../context/GameContext'
import type { Player } from '../lib/bindings/Player'

export default function Lobby() {
  const { mode } = useParams<{ mode: 'create' | 'join' }>()
  const navigate = useNavigate()
  const {
    game,
    playerId,
    username,
    connected,
    messages,
    loading,
    error,
    createGame,
    joinGame,
    connectWebSocket,
    sendMessage,
    clearError,
    leaveGame
  } = useGameContext()

  const [localUsername, setLocalUsername] = useState('')
  const [joinUsername, setJoinUsername] = useState('')
  const [joinGameCode, setJoinGameCode] = useState('')

  const handleCreateGame = async () => {
    await createGame(localUsername)
    setTimeout(() => {
      connectWebSocket()
    }, 100)
  }

  const handleJoinGame = async () => {
    await joinGame(joinUsername, joinGameCode)
    setTimeout(() => {
      connectWebSocket()
    }, 100)
  }

  const handleLeaveGame = () => {
    if (game?.id && username) {
      const playerLeftMessage = {
        game_id: game.id,
        message: {
          type: 'PlayerLeft' as const,
          username: username,
          player_id: playerId || ''
        }
      }
      sendMessage(playerLeftMessage)
    }
    
    setTimeout(() => {
      leaveGame()
      navigate('/')
    }, 100)
  }

  const handleBackToHome = () => {
    navigate('/')
  }

  return (
    <Screen>
      <div className="flex justify-between items-center mb-6 sm:mb-8">
        <Button
          variant="secondary"
          size="small"
          onMouseUp={handleBackToHome}
          className="text-xs sm:text-sm"
        >
          ← Back
        </Button>
        
        {game && (
          <Button
            variant="secondary"
            size="small"
            onMouseUp={handleLeaveGame}
            className="text-xs sm:text-sm text-red-600 hover:text-red-700"
          >
            Leave Game
          </Button>
        )}
      </div>

      <h1 className="text-3xl xs:text-4xl sm:text-5xl md:text-6xl font-bold mb-6 sm:mb-8 font-primary text-center leading-tight">
        yappers.live
      </h1>
      
      {error && (
        <Section variant="error">
          <p className="text-red-700 font-secondary text-sm sm:text-base">{error}</p>
          <Button
            variant="secondary"
            size="small"
            onMouseUp={clearError}
            className="mt-2"
          >
            Dismiss
          </Button>
        </Section>
      )}
      
      {/* Game Creation */}
      {mode === 'create' && !game && (
        <Section title="Create Game">
          <FormRow>
            <Input
              type="text"
              value={localUsername}
              onChange={(e) => setLocalUsername(e.target.value)}
              placeholder="Enter username"
              disabled={loading}
              className="flex-1"
            />
            <Button
              variant="secondary"
              size="medium"
              onMouseUp={handleCreateGame}
              disabled={loading || !localUsername.trim()}
              className="w-full sm:w-auto whitespace-nowrap"
            >
              {loading ? 'Creating...' : 'Create Game'}
            </Button>
          </FormRow>
        </Section>
      )}

      {/* Join Game */}
      {mode === 'join' && !game && (
        <Section title="Join Existing Game">
          <div className="space-y-2">
            <FormRow>
                <Input
                    type="text"
                    value={joinUsername}
                    onChange={(e) => setJoinUsername(e.target.value)}
                    placeholder="Enter username"
                    disabled={loading}
                />
              <Input
                type="text"
                variant="code"
                value={joinGameCode}
                onChange={(e) => setJoinGameCode(e.target.value.toUpperCase())}
                placeholder="Enter code"
                maxLength={6}
                disabled={loading}
              />
              <Button
                variant="secondary"
                size="medium"
                onMouseUp={handleJoinGame}
                disabled={loading || !joinUsername.trim() || !joinGameCode.trim()}
                className="w-full sm:w-auto whitespace-nowrap"
              >
                {loading ? 'Joining...' : 'Join Game'}
              </Button>
            </FormRow>
          </div>
        </Section>
      )}

      {game && (
        <Section title="Game Info">
          <GameCodeDisplay code={game.code} />
          <div className="space-y-1 text-sm sm:text-base mb-4">
            <p className="font-secondary text-pencil break-words"><strong>Game ID:</strong> {game.id}</p>
            <p className="font-secondary text-pencil break-words"><strong>Host:</strong> {game.players.find((p: Player) => p.id === game.host_id)?.username}</p>
          </div>
          <PlayerList 
            players={game.players}
            hostId={game.host_id}
            currentPlayerId={playerId || ''}
            maxPlayers={game.max_players}
          />
        </Section>
      )}

      {/* Chat */}
      {connected && (
        <Section title="Chat">
          <ChatBox 
            messages={messages}
            onSendMessage={sendMessage}
            gameId={game?.id || ''}
            username={username || ''}
          />
        </Section>
      )}
    </Screen>
  )
} 