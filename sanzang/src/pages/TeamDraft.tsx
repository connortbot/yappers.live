import { useNavigate } from 'react-router'
import { Button } from '../components/Button'
import { Screen } from '../components/Screen'
import { Section } from '../components/Section'
import { PlayerList } from '../components/PlayerList'
import { ChatBox } from '../components/ChatBox'
import { useGameContext } from '../context/GameContext'
import type { Player } from '../lib/bindings/Player'

export default function TeamDraft() {
  const navigate = useNavigate()
  const {
    game,
    playerId,
    username,
    connected,
    messages,
    teamDraftState,
    sendMessage,
    leaveGame
  } = useGameContext()

  const handleLeaveGame = () => {
    if (game?.id && username) {
      const playerLeftMessage = {
        type: 'PlayerLeft' as const,
        username: username,
        player_id: playerId || ''
      }
      sendMessage(playerLeftMessage)
    }
    
    setTimeout(() => {
      leaveGame()
      navigate('/')
    }, 100)
  }

  const handleBackToLobby = () => {
    navigate('/lobby/join')
  }

  if (!game || !teamDraftState) {
    return (
      <Screen>
        <div className="text-center">
          <h1 className="text-3xl xs:text-4xl sm:text-5xl md:text-6xl font-bold mb-6 sm:mb-8 font-primary leading-tight">
            yappers.live
          </h1>
          <p className="text-pencil font-secondary mb-4">No active game found</p>
          <Button
            variant="secondary"
            size="medium"
            onMouseUp={() => navigate('/')}
          >
            Back to Home
          </Button>
        </div>
      </Screen>
    )
  }

  return (
    <Screen>
      <div className="flex justify-between items-center mb-6 sm:mb-8">
        <Button
          variant="secondary"
          size="small"
          onMouseUp={handleBackToLobby}
          className="text-xs sm:text-sm"
        >
          ← Back to Lobby
        </Button>
        
        <Button
          variant="secondary"
          size="small"
          onMouseUp={handleLeaveGame}
          className="text-xs sm:text-sm text-red-600 hover:text-red-700"
        >
          Leave Game
        </Button>
      </div>

      <h1 className="text-3xl xs:text-4xl sm:text-5xl md:text-6xl font-bold mb-6 sm:mb-8 font-primary text-center leading-tight">
        Team Draft
      </h1>

      <Section title="Draft Status">
        <div className="space-y-2 text-sm sm:text-base">
          <p className="font-secondary text-pencil">
            <strong>Yapper:</strong> {game.players.find((p: Player) => p.id === teamDraftState.yapper_id)?.username || 'Unknown'}
          </p>
          {teamDraftState.round_data.pool && (
            <p className="font-secondary text-pencil">
              <strong>Pool:</strong> {teamDraftState.round_data.pool}
            </p>
          )}
          {teamDraftState.round_data.competition && (
            <p className="font-secondary text-pencil">
              <strong>Competition:</strong> {teamDraftState.round_data.competition}
            </p>
          )}
          <p className="font-secondary text-pencil">
            <strong>Team Size:</strong> {teamDraftState.round_data.team_size}
          </p>
        </div>
      </Section>

      {connected && (
        <div className="flex gap-2 sm:gap-3 flex-col sm:flex-row">
          <Section title="Chat" className="flex-1">
            <ChatBox 
              messages={messages}
              onSendMessage={sendMessage}
              username={username || ''}
            />
          </Section>
          <Section title="Game" className="flex-1">
            <PlayerList 
              players={game.players}
              hostId={game.host_id}
              currentPlayerId={playerId || ''}
              maxPlayers={game.max_players}
            />
          </Section>
        </div>
      )}
    </Screen>
  )
} 