import type { Player } from '@/lib/types'

interface PlayerListProps {
  players: Player[]
  hostId: string
  currentPlayerId: string
}

export function PlayerList({ players, hostId, currentPlayerId }: PlayerListProps) {
  return (
    <div className="space-y-1 text-sm sm:text-base">
      <p className="font-secondary text-pencil">
        <strong>Players ({players.length}):</strong>
      </p>
      <ul className="ml-4 mt-2 space-y-1">
        {players.map((player) => (
          <li key={player.id} className="flex flex-wrap items-center gap-2 font-secondary text-pencil text-sm sm:text-base">
            <span className="break-words">{player.username}</span>
            {player.id === hostId && (
              <span className="text-xs border-2 border-pencil px-2 py-1 rounded whitespace-nowrap">HOST</span>
            )}
            {player.id === currentPlayerId && (
              <span className="text-xs border-2 border-pencil px-2 py-1 rounded whitespace-nowrap">YOU</span>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}
