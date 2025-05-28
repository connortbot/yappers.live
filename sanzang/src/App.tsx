import { useState, useEffect, useRef } from 'react'

interface Player {
  id: string
  username: string
}

interface LobbyInfo {
  id: string
  code: string
  host_id: string
  players: Player[]
  max_players: number
  created_at: string
}

function App() {
  const [username, setUsername] = useState('')
  const [joinUsername, setJoinUsername] = useState('')
  const [joinLobbyCode, setJoinLobbyCode] = useState('')
  const [lobbyId, setLobbyId] = useState('')
  const [playerId, setPlayerId] = useState('')
  const [message, setMessage] = useState('')
  const [messages, setMessages] = useState<string[]>([])
  const [connected, setConnected] = useState(false)
  const [lobbyInfo, setLobbyInfo] = useState<LobbyInfo | null>(null)
  const wsRef = useRef<WebSocket | null>(null)

  const createLobby = async () => {
    try {
      const response = await fetch('http://localhost:8080/lobby/create', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username }),
      })
      const data: LobbyInfo = await response.json()
      setLobbyId(data.id)
      setLobbyInfo(data)
      const hostPlayer = data.players.find((p: Player) => p.username === username)
      setPlayerId(hostPlayer?.id || username)
    } catch (error) {
      console.error('Error creating lobby:', error)
    }
  }

  const joinLobby = async () => {
    try {
      const response = await fetch('http://localhost:8080/lobby/join', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ 
          username: joinUsername, 
          lobby_code: joinLobbyCode 
        }),
      })
      const data: LobbyInfo = await response.json()
      setLobbyId(data.id)
      setLobbyInfo(data)
      const joinedPlayer = data.players.find((p: Player) => p.username === joinUsername)
      setPlayerId(joinedPlayer?.id || joinUsername)
    } catch (error) {
      console.error('Error joining lobby:', error)
    }
  }

  const connectWebSocket = () => {
    if (!lobbyId || !playerId) return
    
    const ws = new WebSocket(`ws://localhost:8080/ws/${lobbyId}/${playerId}`)
    
    ws.onopen = () => {
      console.log('Connected to websocket')
      setConnected(true)
    }

    ws.onmessage = (event) => {
      try {
        const wsMessage = JSON.parse(event.data)
        
        // Check if it's a structured WebSocket message
        if (wsMessage.message && wsMessage.message.type) {
          switch (wsMessage.message.type) {
            case 'PlayerJoined':
              // Update lobby info by adding the new player
              setLobbyInfo((prev: LobbyInfo | null) => {
                if (!prev) return prev
                
                // Check if player already exists to avoid duplicates
                const playerExists = prev.players.some((p: Player) => p.id === wsMessage.message.player_id)
                if (playerExists) return prev
                
                return {
                  ...prev,
                  players: [
                    ...prev.players,
                    {
                      id: wsMessage.message.player_id,
                      username: wsMessage.message.username
                    }
                  ]
                }
              })
              
              // Add join message to chat
              setMessages(prev => [...prev, `🎉 ${wsMessage.message.username} joined the lobby!`])
              break
              
            case 'PlayerLeft':
              // Update lobby info by removing the player
              setLobbyInfo((prev: LobbyInfo | null) => {
                if (!prev) return prev
                
                return {
                  ...prev,
                  players: prev.players.filter((p: Player) => p.id !== wsMessage.message.player_id)
                }
              })
              
              // Add leave message to chat
              setMessages(prev => [...prev, `👋 ${wsMessage.message.username} left the lobby`])
              break
              
            case 'ChatMessage':
              // Add chat message
              setMessages(prev => [...prev, `${wsMessage.message.username}: ${wsMessage.message.message}`])
              break
              
            case 'GameStarted':
              // Add game started message
              setMessages(prev => [...prev, `🎮 Game started: ${wsMessage.message.game_type}`])
              break
              
            default:
              // Unknown message type, just display as text
              setMessages(prev => [...prev, event.data])
          }
        } else {
          // Not a structured message, treat as plain text
          setMessages(prev => [...prev, event.data])
        }
      } catch (error) {
        // Failed to parse JSON, treat as plain text message
        setMessages(prev => [...prev, event.data])
      }
    }

    ws.onclose = () => {
      console.log('Disconnected from websocket')
      setConnected(false)
    }

    wsRef.current = ws
  }

  const disconnect = () => {
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
      setConnected(false)
      setMessages([])
    }
  }

  const sendMessage = () => {
    if (wsRef.current && message) {
      wsRef.current.send(message)
      setMessage('')
    }
  }

  return (
    <div className="p-4 max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">Yappers Websocket Test</h1>
      
      {/* Lobby Creation */}
      <div className="mb-8 p-4 border rounded-lg">
        <h2 className="text-xl font-semibold mb-4">Create Lobby</h2>
        <div className="flex gap-2">
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Enter username"
            className="border rounded px-2 py-1 flex-1"
          />
          <button
            onClick={createLobby}
            className="bg-blue-500 text-white px-4 py-1 rounded hover:bg-blue-600"
          >
            Create Lobby
          </button>
        </div>
      </div>

      <div className="mb-8 p-4 border rounded-lg">
        <h2 className="text-xl font-semibold mb-4">Join Existing Lobby</h2>
        <div className="flex gap-2 mb-2">
          <input
            type="text"
            value={joinUsername}
            onChange={(e) => setJoinUsername(e.target.value)}
            placeholder="Enter username"
            className="border rounded px-2 py-1 flex-1"
          />
          <input
            type="text"
            value={joinLobbyCode}
            onChange={(e) => setJoinLobbyCode(e.target.value.toUpperCase())}
            placeholder="Enter 6-digit code (e.g. ABC123)"
            maxLength={6}
            className="border rounded px-2 py-1 flex-1 font-mono tracking-widest text-center"
          />
          <button
            onClick={joinLobby}
            className="bg-green-500 text-white px-4 py-1 rounded hover:bg-green-600"
          >
            Join Lobby
          </button>
        </div>
      </div>

      {lobbyInfo && (
        <div className="mb-8 p-4 border rounded-lg bg-gray-50">
          <h2 className="text-xl font-semibold mb-4">Lobby Info</h2>
          <div className="mb-4 p-3 bg-blue-100 rounded-lg border-2 border-blue-300">
            <p className="text-sm text-blue-700 mb-1">Share this code with friends:</p>
            <p className="text-3xl font-bold text-blue-800 tracking-widest">{lobbyInfo.code}</p>
          </div>
          <p><strong>Lobby ID:</strong> {lobbyInfo.id}</p>
          <p><strong>Host:</strong> {lobbyInfo.players.find((p: Player) => p.id === lobbyInfo.host_id)?.username}</p>
          <p><strong>Players ({lobbyInfo.players.length}/{lobbyInfo.max_players}):</strong></p>
          <ul className="ml-4">
            {lobbyInfo.players.map((player: Player) => (
              <li key={player.id} className="flex items-center gap-2">
                <span>{player.username}</span>
                {player.id === lobbyInfo.host_id && <span className="text-xs bg-blue-100 px-2 py-1 rounded">HOST</span>}
                {player.id === playerId && <span className="text-xs bg-green-100 px-2 py-1 rounded">YOU</span>}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Websocket Connection */}
      <div className="mb-8 p-4 border rounded-lg">
        <h2 className="text-xl font-semibold mb-4">Websocket Connection</h2>
        <div className="flex gap-2">
          <button
            onClick={connectWebSocket}
            disabled={!lobbyId || !playerId || connected}
            className={`px-4 py-1 rounded ${
              connected
                ? 'bg-green-500'
                : 'bg-blue-500 hover:bg-blue-600'
            } text-white disabled:bg-gray-400`}
          >
            {connected ? 'Connected' : 'Connect to Websocket'}
          </button>
          {connected && (
            <button
              onClick={disconnect}
              className="bg-red-500 text-white px-4 py-1 rounded hover:bg-red-600"
            >
              Disconnect
            </button>
          )}
        </div>
        {lobbyId && playerId && (
          <p className="mt-2 text-sm text-gray-600">
            Ready to connect to lobby {lobbyId} as player {playerId}
          </p>
        )}
      </div>

      {/* Chat */}
      {connected && (
        <div className="p-4 border rounded-lg">
          <h2 className="text-xl font-semibold mb-4">Chat</h2>
          <div className="mb-4 h-48 overflow-y-auto border rounded p-2 bg-gray-50">
            {messages.map((msg, index) => (
              <div key={index} className="mb-1">
                {msg}
              </div>
            ))}
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Type a message"
              className="border rounded px-2 py-1 flex-1"
              onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
            />
            <button
              onClick={sendMessage}
              className="bg-blue-500 text-white px-4 py-1 rounded hover:bg-blue-600"
            >
              Send
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default App
