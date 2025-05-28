import { useState, useEffect, useRef } from 'react'

function App() {
  const [username, setUsername] = useState('')
  const [joinUsername, setJoinUsername] = useState('')
  const [joinLobbyId, setJoinLobbyId] = useState('')
  const [lobbyId, setLobbyId] = useState('')
  const [playerId, setPlayerId] = useState('')
  const [message, setMessage] = useState('')
  const [messages, setMessages] = useState<string[]>([])
  const [connected, setConnected] = useState(false)
  const [lobbyInfo, setLobbyInfo] = useState<any>(null)
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
      const data = await response.json()
      setLobbyId(data.id)
      setLobbyInfo(data)
      const hostPlayer = data.players.find((p: any) => p.username === username)
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
          lobby_id: joinLobbyId 
        }),
      })
      const data = await response.json()
      setLobbyId(data.id)
      setLobbyInfo(data)
      const joinedPlayer = data.players.find((p: any) => p.username === joinUsername)
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
      setMessages(prev => [...prev, event.data])
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
            value={joinLobbyId}
            onChange={(e) => setJoinLobbyId(e.target.value)}
            placeholder="Enter lobby ID"
            className="border rounded px-2 py-1 flex-1"
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
          <p><strong>Lobby ID:</strong> {lobbyInfo.id}</p>
          <p><strong>Host:</strong> {lobbyInfo.players.find((p: any) => p.id === lobbyInfo.host_id)?.username}</p>
          <p><strong>Players ({lobbyInfo.players.length}/{lobbyInfo.max_players}):</strong></p>
          <ul className="ml-4">
            {lobbyInfo.players.map((player: any) => (
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
