interface GameListProps {
  gameIds: string[]
  count: number
  onGameClick: (gameId: string) => void
}

export function GameList({ gameIds, count, onGameClick }: GameListProps) {
  return (
    <div className="space-y-3">
      <p className="font-primary text-lg sm:text-xl font-bold text-pencil">
        Games ({count})
      </p>
      
      {gameIds.length === 0 ? (
        <p className="font-secondary text-pencil text-sm sm:text-base italic">
          No games found
        </p>
      ) : (
        <div className="space-y-2">
          {gameIds.map((gameId) => (
            <div
              key={gameId}
              onClick={() => onGameClick(gameId)}
              className="p-3 bg-background border-2 border-pencil rounded-lg cursor-pointer hover:bg-gray-50 transition-colors font-secondary text-pencil text-sm sm:text-base break-all"
            >
              {gameId}
            </div>
          ))}
        </div>
      )}
    </div>
  )
} 