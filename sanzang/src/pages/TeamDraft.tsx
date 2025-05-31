import { useNavigate } from 'react-router'
import { useState, useEffect } from 'react'
import { IoCheckmark } from 'react-icons/io5'
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
    createCompetitionMessage,
    createStartDraftMessage
  } = useGameContext()

  const [poolInput, setPoolInput] = useState('')
  const [competitionInput, setCompetitionInput] = useState('')
  const [showPlayerSelection, setShowPlayerSelection] = useState(false)
  const [selectedPlayerId, setSelectedPlayerId] = useState<string | null>(null)

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
    if (poolInput && competitionInput) {
      setShowPlayerSelection(true)
    }
  }

  const handlePlayerSelect = (playerId: string) => {
    setSelectedPlayerId(playerId)
  }

  const handleStartDraft = () => {
    if (selectedPlayerId) {
      const startDraftMessage = createStartDraftMessage(selectedPlayerId)
      sendMessage(startDraftMessage)
      setShowPlayerSelection(false)
      setSelectedPlayerId(null)
    }
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

  // StartDraft handler
  useEffect(() => {
    if (latestEvent?.type === 'TeamDraft' && latestEvent.msg_type === 'StartDraft' && teamDraftState?.round_data.current_drafter_id) {
      console.log('StartDraft event received')
    }
  }, [latestEvent, teamDraftState?.round_data.current_drafter_id])

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
      if (showPlayerSelection && isYapper) {
        return (
          <Section title="Choose the first drafter:">
            <div className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {game.players.map((player) => (
                  player.id !== playerId && (
                    <Button
                      key={player.id}
                      variant={'primary'}
                      size="medium"
                      onMouseUp={() => handlePlayerSelect(player.id)}
                      className="w-full"
                    >
                      <span className="flex items-center justify-center gap-2">
                        {player.username}
                        {selectedPlayerId === player.id && <IoCheckmark className="w-5 h-5" />}
                      </span>
                    </Button>
                  )
                ))}
              </div>
              
              {selectedPlayerId && (
                <div className="text-center">
                  <Button
                    variant="primary"
                    size="large"
                    onMouseUp={handleStartDraft}
                  >
                    Start Draft!
                  </Button>
                </div>
              )}
            </div>
          </Section>
        )
      }

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
                disabled={!poolInput || !competitionInput}
              >
                Let's Begin!
              </Button>
            )}
          </div>
        </Section>
      )
    }

    if (teamDraftState.phase === 'Drafting') {
      const currentDrafterId = teamDraftState.round_data.current_drafter_id
      const currentDrafter = game.players.find(p => p.id === currentDrafterId)
      const isDrafting = currentDrafterId === playerId

      return (
        <Section title="Drafting Phase">
          <div className="text-center">
            {isDrafting ? (
              <p className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                I am drafting
              </p>
            ) : (
              <p className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                {currentDrafter?.username || 'Someone'} is thinking!
              </p>
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
        <>
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
        </>
      )}
    </Screen>
  )
} 