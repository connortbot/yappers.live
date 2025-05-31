import { useState } from 'react'
import { Screen } from '../components/Screen'
import { Section } from '../components/Section'
import { Input } from '../components/Input'
import { Button } from '../components/Button'
import { FormRow } from '../components/FormRow'
import { GameList } from '../components/GameList'
import { useAdminAPI } from '../hooks/useAdminAPI'
import type { Game } from '../lib/bindings/Game'
import type { Player } from '../lib/bindings/Player'

export default function Admin() {
  const { listGames, getGame } = useAdminAPI()
  
  const [password, setPassword] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const [gameIds, setGameIds] = useState<string[]>([])
  const [gameCount, setGameCount] = useState(0)
  const [selectedGame, setSelectedGame] = useState<Game | null>(null)
  const [gameLoading, setGameLoading] = useState(false)

  const handleFetchGames = async () => {
    if (!password.trim()) return
    
    setLoading(true)
    setError(null)
    
    try {
      const result = await listGames(password.trim())
      
      if (result.error) {
        setError(result.error.message || 'Failed to fetch games')
        setGameIds([])
        setGameCount(0)
        return
      }
      
      if (result.data) {
        setGameIds(result.data.game_ids)
        setGameCount(result.data.count)
      }
    } catch (error) {
      console.error('Error fetching games:', error)
      setError('Failed to fetch games. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  const handleGameClick = async (gameId: string) => {
    if (!password.trim()) return
    
    setGameLoading(true)
    setError(null)
    
    try {
      const result = await getGame(gameId, password.trim())
      
      if (result.error) {
        setError(result.error.message || 'Failed to fetch game details')
        setSelectedGame(null)
        return
      }
      
      if (result.data) {
        setSelectedGame(result.data.game)
      }
    } catch (error) {
      console.error('Error fetching game details:', error)
      setError('Failed to fetch game details. Please try again.')
    } finally {
      setGameLoading(false)
    }
  }

  const clearError = () => {
    setError(null)
  }

  return (
    <Screen>
      <h1 className="text-3xl xs:text-4xl sm:text-5xl md:text-6xl font-bold mb-6 sm:mb-8 font-primary text-center leading-tight">
        Admin Panel
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

      <Section title="Authentication">
        <FormRow>
          <Input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Enter admin password"
            className="flex-1"
          />
          <Button
            variant="primary"
            size="medium"
            onMouseUp={handleFetchGames}
            disabled={loading || !password.trim()}
            className="w-full sm:w-auto whitespace-nowrap"
          >
            {loading ? 'Fetching...' : 'Fetch'}
          </Button>
        </FormRow>
      </Section>

      {gameCount > 0 && (
        <Section title="Games">
          <GameList
            gameIds={gameIds}
            count={gameCount}
            onGameClick={handleGameClick}
          />
        </Section>
      )}

      {selectedGame && (
        <Section title="Game Details">
          {gameLoading ? (
            <p className="font-secondary text-pencil text-sm sm:text-base">Loading game details...</p>
          ) : (
            <div className="space-y-3">
              <div className="space-y-1 text-sm sm:text-base">
                <p className="font-secondary text-pencil break-words">
                  <strong>Game ID:</strong> {selectedGame.id}
                </p>
                <p className="font-secondary text-pencil break-words">
                  <strong>Code:</strong> {selectedGame.code}
                </p>
                <p className="font-secondary text-pencil break-words">
                  <strong>Host:</strong> {selectedGame.players.find((p: Player) => p.id === selectedGame.host_id)?.username}
                </p>
                <p className="font-secondary text-pencil">
                  <strong>Players:</strong> {selectedGame.players.length}/{selectedGame.max_players}
                </p>
                <p className="font-secondary text-pencil">
                  <strong>Created:</strong> {new Date(selectedGame.created_at * 1000).toLocaleString()}
                </p>
              </div>
              
              <div className="space-y-1">
                <p className="font-secondary text-pencil font-bold">Player List:</p>
                <ul className="ml-4 space-y-1">
                  {selectedGame.players.map((player: Player) => (
                    <li key={player.id} className="flex items-center gap-2 font-secondary text-pencil text-sm sm:text-base">
                      <span className="break-words">{player.username}</span>
                      {player.id === selectedGame.host_id && (
                        <span className="text-xs border-2 border-pencil px-2 py-1 rounded">HOST</span>
                      )}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          )}
        </Section>
      )}
    </Screen>
  )
} 