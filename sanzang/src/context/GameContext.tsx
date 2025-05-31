import React, { createContext, useContext, useState, useRef, useCallback, useEffect } from 'react'
import { useGameAPI } from '../hooks/useGameAPI'
import type { Game } from '../lib/bindings/Game'
import type { Player } from '../lib/bindings/Player'
import type { WebSocketMessage } from '../lib/bindings/WebSocketMessage'
import type { GameMessage } from '../lib/bindings/GameMessage'
import type { TeamDraftManager } from '../lib/bindings/TeamDraftManager'
import { createWukongWebSocket } from '../lib/wukongClient'
import { GameStartedMessage } from '../lib/bindings/GameStartedMessage'

interface GameContextState {
  game: Game | null
  playerId: string | null
  username: string | null
  authToken: string | null
  
  connected: boolean
  connecting: boolean
  
  messages: string[]
  latestEvent: GameMessage | null
  
  // Game mode states
  teamDraftState: TeamDraftManager | null
  
  loading: boolean
  error: string | null
  
  createGame: (username: string) => Promise<void>
  joinGame: (username: string, gameCode: string) => Promise<void>
  connectWebSocket: () => void
  disconnect: () => void
  sendMessage: (message: GameMessage) => void
  clearError: () => void
  leaveGame: () => void
}

const GameContext = createContext<GameContextState | null>(null)

export function useGameContext() {
  const context = useContext(GameContext)
  if (!context) {
    throw new Error('useGameContext must be used within a GameProvider')
  }
  return context
}

interface GameProviderProps {
  children: React.ReactNode
}

export function GameProvider({ children }: GameProviderProps) {
  const { createGame: createGameAPI, joinGame: joinGameAPI } = useGameAPI()
  
  const [game, setGame] = useState<Game | null>(null)
  const [playerId, setPlayerId] = useState<string | null>(null)
  const [username, setUsername] = useState<string | null>(null)
  const [authToken, setAuthToken] = useState<string | null>(null)
  
  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [messages, setMessages] = useState<string[]>([])
  const [latestEvent, setLatestEvent] = useState<GameMessage | null>(null)
  
  const [teamDraftState, setTeamDraftState] = useState<TeamDraftManager | null>(null)
  
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const wsRef = useRef<WebSocket | null>(null)

  const handleGameMessage = useCallback((message: GameMessage) => {
    switch (message.type) {
      case 'PlayerJoined':
        setGame((prev: Game | null) => {
          if (!prev) return prev
          
          const playerExists = prev.players.some((p: Player) => p.id === message.player_id)
          if (playerExists) return prev
          
          return {
            ...prev,
            players: [
              ...prev.players,
              {
                id: message.player_id,
                username: message.username
              }
            ]
          }
        })
        
        setMessages(prev => [...prev, `🎉 ${message.username} joined the game!`])
        break
        
      case 'PlayerLeft':
        setGame((prev: Game | null) => {
          if (!prev) return prev
          
          return {
            ...prev,
            players: prev.players.filter((p: Player) => p.id !== message.player_id)
          }
        })
        
        setMessages(prev => [...prev, `👋 ${message.username} left the game`])
        break

      case 'PlayerDisconnected':
        setGame((prev: Game | null) => {
          if (!prev) return prev
          
          return {
            ...prev,
            players: prev.players.filter((p: Player) => p.id !== message.player_id)
          }
        })
        
        setMessages(prev => [...prev, `🔌 ${message.username} disconnected`])
        break
        
      case 'ChatMessage':
        setMessages(prev => [...prev, `${message.username}: ${message.message}`])
        break
        
      case 'GameStarted':
        setLatestEvent(message)
        setMessages(prev => [...prev, `🎮 Game started: ${(message as GameStartedMessage).game_type.type}`])
        
        const gameStartedMessage = message as GameStartedMessage
        if (gameStartedMessage.game_type.type === 'TeamDraft' && gameStartedMessage.initial_team_draft_state) {
          setTeamDraftState(gameStartedMessage.initial_team_draft_state)
        }
        break
        
      default:
        console.warn('Unknown message type:', message)
    }
  }, [])

  const connectWebSocketWithGameData = useCallback((gameData: Game, playerIdData: string) => {
    if (connected || connecting) return
    
    setConnecting(true)
    setError(null)
    
    const ws = createWukongWebSocket(`${gameData.id}/${playerIdData}`)
    
    ws.onopen = () => {
      console.log('Connected to websocket')
      setConnected(true)
      setConnecting(false)
    }

    ws.onmessage = (event) => {
      try {
        const wsMessage: WebSocketMessage = JSON.parse(event.data)
        handleGameMessage(wsMessage.message)
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
        setMessages(prev => [...prev, event.data])
      }
    }

    ws.onclose = () => {
      console.log('Disconnected from websocket')
      setConnected(false)
      setConnecting(false)
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
      setError('WebSocket connection failed')
      setConnecting(false)
    }

    wsRef.current = ws
  }, [connected, connecting, handleGameMessage])

  const connectWebSocket = useCallback(() => {
    if (!game?.id || !playerId || connected || connecting) return
    
    connectWebSocketWithGameData(game, playerId)
  }, [game?.id, playerId, connected, connecting, connectWebSocketWithGameData])

  const createGame = useCallback(async (usernameInput: string) => {
    if (!usernameInput.trim()) {
      setError('Please enter a username')
      return
    }
    
    setLoading(true)
    setError(null)
    
    try {
      const result = await createGameAPI({ username: usernameInput.trim() })
      
      if (result.error) {
        setError(result.error.message || 'Failed to create game')
        return
      }
      
      if (result.data) {
        const data = result.data
        const trimmedUsername = usernameInput.trim()
        const hostPlayer = data.game.players.find((p: Player) => p.username === trimmedUsername)
        const hostPlayerId = hostPlayer?.id || trimmedUsername
        
        setGame(data.game)
        setUsername(trimmedUsername)
        setPlayerId(hostPlayerId)
        setAuthToken(data.auth_token)
        connectWebSocketWithGameData(data.game, hostPlayerId)
      }
    } catch (error) {
      console.error('Error creating game:', error)
      setError('Failed to create game. Please try again.')
    } finally {
      setLoading(false)
    }
  }, [createGameAPI, connectWebSocketWithGameData])

  const joinGame = useCallback(async (usernameInput: string, gameCode: string) => {
    if (!usernameInput.trim()) {
      setError('Please enter a username')
      return
    }
    
    if (!gameCode.trim()) {
      setError('Please enter a game code')
      return
    }
    
    setLoading(true)
    setError(null)
    
    try {
      const result = await joinGameAPI({ 
        username: usernameInput.trim(), 
        game_code: gameCode.trim() 
      })
      
      if (result.error) {
        setError(result.error.message || 'Failed to join game')
        return
      }
      
      if (result.data) {
        const data = result.data
        const trimmedUsername = usernameInput.trim()
        const joinedPlayer = data.game.players.find((p: Player) => p.username === trimmedUsername)
        const joinedPlayerId = joinedPlayer?.id || trimmedUsername
        
        setGame(data.game)
        setUsername(trimmedUsername)
        setPlayerId(joinedPlayerId)
        setAuthToken(data.auth_token)
        
        connectWebSocketWithGameData(data.game, joinedPlayerId)
      }
    } catch (error) {
      console.error('Error joining game:', error)
      setError('Failed to join game. Please try again.')
    } finally {
      setLoading(false)
    }
  }, [joinGameAPI, connectWebSocketWithGameData])

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
      setConnected(false)
      setConnecting(false)
      setMessages([])
    }
  }, [])

  const sendMessage = (message: GameMessage) => {
    if (wsRef.current && message) {
      wsRef.current.send(JSON.stringify({
        game_id: game?.id,
        auth_token: authToken,
        player_id: playerId,
        message: message
      }))
    }
  }

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  const leaveGame = useCallback(() => {
    disconnect()
    setGame(null)
    setPlayerId(null)
    setUsername(null)
    setMessages([])
    setError(null)
  }, [disconnect])

  useEffect(() => {
    return () => {
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [])

  const contextValue: GameContextState = {
    game,
    playerId,
    username,
    authToken,
    connected,
    connecting,
    messages,
    teamDraftState,
    loading,
    latestEvent,
    error,
    createGame,
    joinGame,
    connectWebSocket,
    disconnect,
    sendMessage,
    clearError,
    leaveGame
  }

  return (
    <GameContext.Provider value={contextValue}>
      {children}
    </GameContext.Provider>
  )
} 