import { useNavigate } from 'react-router'
import { useState, useEffect } from 'react'
import { Button } from '../components/Button'
import { Screen } from '../components/Screen'
import { Section } from '../components/Section'
import { PlayerList } from '../components/PlayerList'
import { ChatBox } from '../components/ChatBox'
import { Input } from '../components/Input'
import { FormRow } from '../components/FormRow'
import { useGameContext } from '../context/GameContext'

export default function TeamDraft() {
  const navigate = useNavigate()
  const {
    game,
    playerId,
    username,
    connected,
    messages,
    latestEvent,
    teamDraftState,
    sendMessage,
    leaveGame,
    createPoolMessage,
    createCompetitionMessage
  } = useGameContext()

  const [poolInput, setPoolInput] = useState('')
  const [competitionInput, setCompetitionInput] = useState('')

  const isYapper = playerId === teamDraftState?.yapper_id

  const handlePoolChange = (value: string) => {
    setPoolInput(value)
    if (value) {
      const poolMessage = createPoolMessage(value)
      sendMessage(poolMessage)
    }
  }

  const handleCompetitionChange = (value: string) => {
    setCompetitionInput(value)
    if (value) {
      const competitionMessage = createCompetitionMessage(value)
      sendMessage(competitionMessage)
    }
  }

  const handleConfirm = () => {
    
  }

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

  useEffect(() => {
    if (latestEvent) {
      if (latestEvent.type === 'TeamDraft') {
        switch (latestEvent.msg_type) {
          case 'SetPool':
            setPoolInput(latestEvent.pool)
            break
          case 'SetCompetition':
            setCompetitionInput(latestEvent.competition)
            break
        }
      }
    }
  }, [latestEvent])

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

  const renderDraftSection = () => {
    if (teamDraftState.phase === 'YapperChoosing') {
      return (
        <Section title={isYapper ? "What is this round all about?" : "The Yapper is choosing!"}>
          <div className="space-y-4 text-center">
            <p className="font-secondary text-pencil text-base sm:text-lg italic">
              Choose someone/something that is...
            </p>
            
            {isYapper ? (
              <FormRow>
                <Input
                  type="text"
                  value={poolInput}
                  onChange={(e) => handlePoolChange(e.target.value)}
                  placeholder="Enter pool description"
                  className="flex-1"
                />
              </FormRow>
            ) : (
              <div className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                {teamDraftState.round_data.pool || '...'}
              </div>
            )}

            <p className="font-secondary text-pencil text-base sm:text-lg italic">
              and is competing to win at...
            </p>

            {isYapper ? (
              <FormRow>
                <Input
                  type="text"
                  value={competitionInput}
                  onChange={(e) => handleCompetitionChange(e.target.value)}
                  placeholder="Enter competition description"
                  className="flex-1"
                />
              </FormRow>
            ) : (
              <div className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                {teamDraftState.round_data.competition || '...'}
              </div>
            )}

            {isYapper && (
              <Button
                variant="secondary"
                size="medium"
                onMouseUp={handleConfirm}
              >
                Let's Begin!
              </Button>
            )}
          </div>
        </Section>
      )
    }
    
    return null
  }

  return (
    <Screen>
      <div className="flex justify-between items-center mb-6 sm:mb-8">
        <h1 className="text-2xl xs:text-3xl sm:text-4xl md:text-5xl font-bold font-primary leading-tight">
          Team Draft
        </h1>
        <Button
          variant="secondary"
          size="small"
          onMouseUp={handleLeaveGame}
          className="text-xs sm:text-sm text-red-600 hover:text-red-700"
        >
          Leave Game
        </Button>
      </div>

      {renderDraftSection()}

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