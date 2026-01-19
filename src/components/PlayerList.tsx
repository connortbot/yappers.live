import type { Player } from '@/lib/types'

const PLAYER_TIMEOUT_MS = 30000 // 30 seconds

function isPlayerOnline(player: Player): boolean {
  return Date.now() - player.lastSeenAt < PLAYER_TIMEOUT_MS
}

interface PlayerListProps {
  players: Player[]
  hostId: string
  currentPlayerId: string
  showOnlineStatus?: boolean
}

export function PlayerList({ players, hostId, currentPlayerId, showOnlineStatus = false }: PlayerListProps) {
  const onlineCount = showOnlineStatus ? players.filter(isPlayerOnline).length : players.length

  return (
    <div className="space-y-1 text-sm sm:text-base">
      <p className="font-secondary text-pencil">
        <strong>Players ({showOnlineStatus ? `${onlineCount}/${players.length}` : players.length}):</strong>
      </p>
      <ul className="ml-4 mt-2 space-y-1">
        {players.map((player) => {
          const online = isPlayerOnline(player)
          return (
            <li
              key={player.id}
              className={`flex flex-wrap items-center gap-2 font-secondary text-sm sm:text-base ${
                showOnlineStatus && !online ? 'text-gray-400' : 'text-pencil'
              }`}
            >
              {showOnlineStatus && (
                <span className={`w-2 h-2 rounded-full ${online ? 'bg-green-500' : 'bg-gray-300'}`} />
              )}
              <span className="break-words">{player.username}</span>
              {player.id === hostId && (
                <span className="text-xs border-2 border-pencil px-2 py-1 rounded whitespace-nowrap">HOST</span>
              )}
              {player.id === currentPlayerId && (
                <span className="text-xs border-2 border-pencil px-2 py-1 rounded whitespace-nowrap">YOU</span>
              )}
            </li>
          )
        })}
      </ul>
    </div>
  )
}
