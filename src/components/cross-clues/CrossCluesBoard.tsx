'use client'

import type { CrossCluesState } from '@/lib/types'

interface CrossCluesBoardProps {
  crossClues: CrossCluesState
  onCellClick?: (coordinate: string) => void
  highlightedCell?: string | null
  selectedCell?: string | null
}

const COLUMNS = ['A', 'B', 'C', 'D', 'E']
const ROWS = ['1', '2', '3', '4', '5']

export function CrossCluesBoard({
  crossClues,
  onCellClick,
  highlightedCell,
  selectedCell,
}: CrossCluesBoardProps) {
  const { grid, rowWords, colWords } = crossClues

  const getCellStatus = (coordinate: string): 'filled' | 'discarded' | 'available' => {
    return grid[coordinate] || 'available'
  }

  const getCellClasses = (coordinate: string): string => {
    const status = getCellStatus(coordinate)
    const isHighlighted = highlightedCell === coordinate
    const isSelected = selectedCell === coordinate

    let base = 'w-full aspect-square border-2 rounded-lg transition-all flex items-center justify-center font-bold text-lg'

    if (status === 'filled') {
      base += ' bg-green-200 border-green-500 text-green-700'
    } else if (status === 'discarded') {
      base += ' bg-red-100 border-red-300 text-red-400'
    } else if (isSelected) {
      base += ' bg-amber-200 border-amber-500 text-amber-700'
    } else if (isHighlighted) {
      base += ' bg-blue-100 border-blue-400 text-blue-600'
    } else {
      base += ' bg-white border-gray-300 text-gray-400 hover:border-gray-400'
    }

    if (onCellClick && status === 'available') {
      base += ' cursor-pointer'
    }

    return base
  }

  return (
    <div className="w-full max-w-md mx-auto">
      {/* Column headers */}
      <div className="grid grid-cols-6 gap-1 mb-1">
        <div /> {/* Empty corner */}
        {COLUMNS.map((col, idx) => (
          <div key={col} className="text-center">
            <div className="font-bold text-sm text-pencil">{col}</div>
            <div className="text-xs text-pencil truncate" title={colWords[idx]}>
              {colWords[idx]}
            </div>
          </div>
        ))}
      </div>

      {/* Grid rows */}
      {ROWS.map((row, rowIdx) => (
        <div key={row} className="grid grid-cols-6 gap-1 mb-1">
          {/* Row header */}
          <div className="flex items-center justify-end pr-2">
            <div className="text-right">
              <div className="font-bold text-sm text-pencil">{row}</div>
              <div className="text-xs text-pencil truncate max-w-[60px]" title={rowWords[rowIdx]}>
                {rowWords[rowIdx]}
              </div>
            </div>
          </div>

          {/* Cells */}
          {COLUMNS.map((col) => {
            const coordinate = `${col}${row}`
            const status = getCellStatus(coordinate)

            return (
              <button
                key={coordinate}
                type="button"
                className={getCellClasses(coordinate)}
                onClick={() => onCellClick && status === 'available' && onCellClick(coordinate)}
                disabled={!onCellClick || status !== 'available'}
              >
                {status === 'filled' && '✓'}
                {status === 'discarded' && '✗'}
              </button>
            )
          })}
        </div>
      ))}
    </div>
  )
}
