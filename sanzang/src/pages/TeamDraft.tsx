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
import { DraftPick } from '../lib/bindings/DraftPick'
import { TeamDraftMessage } from '../lib/bindings/TeamDraftMessage'
import { GameMessage } from '../lib/bindings/GameMessage'

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
  
  const [timeLeft, setTimeLeft] = useState(0)
  const [timerMessage, setTimerMessage] = useState('')
  const [timerEndTimestamp, setTimerEndTimestamp] = useState<number | null>(null)
  
  const [draftInput, setDraftInput] = useState('')
  const [currentPick, setCurrentPick] = useState('')
  const [currentPickDrafter, setCurrentPickDrafter] = useState('')

  const isYapper = playerId === teamDraftState?.yapper_id

  useEffect(() => {
    if (timerEndTimestamp) {
      const updateTimer = () => {
        const now = Date.now()
        const remaining = Math.max(0, Math.ceil((timerEndTimestamp - now) / 1000))
        setTimeLeft(remaining)
        
        if (remaining > 0) {
          setTimeout(updateTimer, 100)
        }
      }
      updateTimer()
    }
  }, [timerEndTimestamp])

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }

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
    }
  }

  const handleDraftSubmit = () => {
    if (draftInput.trim() && playerId) {
      const draftPickMessage = {
        type: 'TeamDraft',
        msg_type: 'DraftPick',
        drafter_id: playerId,
        pick: draftInput.trim()
      } as GameMessage & TeamDraftMessage & DraftPick
      sendMessage(draftPickMessage)
      setDraftInput('')
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
          case 'DraftPick':
            setCurrentPick(latestEvent.pick)
            setCurrentPickDrafter(latestEvent.drafter_id)
            break
          case 'NextDrafter':
            setCurrentPick('')
            setCurrentPickDrafter(latestEvent.drafter_id)
            break
        }
      } else if (latestEvent.type === 'HaltTimer') {
        if (latestEvent.reason?.TeamDraft === 'YapperStartingDraft') {
          setTimerMessage('The draft begins in...')
          setShowPlayerSelection(false)
          setSelectedPlayerId(null)
        } else if (latestEvent.reason?.TeamDraft === 'DraftPickShowcase') {
          setTimerMessage('Next drafter in...')
        } else {}
        setTimerEndTimestamp(Number(latestEvent.end_timestamp_ms))
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

  const renderTimer = () => {
    return (
      <div className="flex items-center justify-center gap-2 mt-6 pt-0">
        <div className={`text-sm font-secondary text-pencil italic transition-all duration-500 ease-in-out ${
          (timeLeft > 0) ? 'opacity-100 translate-x-0' : 'opacity-25 translate-x-2'
        }`}>
          {timerMessage}
        </div>
        <div className={`text-2xl font-primary font-bold text-pencil transition-opacity duration-300 ${
          timeLeft > 0 ? 'opacity-100' : 'opacity-30'
        }`}>
          {formatTime(timeLeft)}
        </div>
      </div>
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
            {renderTimer()}
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
          {renderTimer()}
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
            {currentPick ? (
              <div className="space-y-2">
                <p className="font-secondary text-pencil text-base sm:text-lg">
                  {currentPickDrafter === playerId ? 'You chose:' : `${game.players.find(p => p.id === currentPickDrafter)?.username || 'Someone'} chose:`}
                </p>
                <p className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                  {currentPick}
                </p>
              </div>
            ) : isDrafting ? (
              <div className="space-y-4">
                <p className="font-secondary text-pencil text-base sm:text-lg">
                  Who/what will you pick?
                </p>
                <FormRow>
                  <Input
                    type="text"
                    value={draftInput}
                    onChange={(e) => setDraftInput(e.target.value)}
                    placeholder="Enter your pick"
                    className="flex-1 font-primary"
                    onKeyUp={(e) => {
                      if (e.key === 'Enter') {
                        handleDraftSubmit()
                      }
                    }}
                  />
                </FormRow>
                <Button
                  variant="primary"
                  size="medium"
                  onMouseUp={handleDraftSubmit}
                  disabled={!draftInput.trim()}
                >
                  Submit Pick
                </Button>
              </div>
            ) : (
              <p className="text-2xl sm:text-3xl font-bold font-primary text-pencil">
                {currentDrafter?.username || 'Someone'} is thinking!
              </p>
            )}
          </div>
          {renderTimer()}
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