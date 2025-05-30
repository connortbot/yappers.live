interface GameCodeDisplayProps {
  code: string
  label?: string
}

export function GameCodeDisplay({ code, label = "Share this code with friends:" }: GameCodeDisplayProps) {
  return (
    <div className="mb-4 p-3 bg-blue-100 rounded-lg border-2 border-pencil">
      <p className="text-xs sm:text-sm text-pencil mb-1 font-secondary">{label}</p>
      <p className="text-2xl sm:text-3xl font-bold text-pencil tracking-widest font-primary break-all">{code}</p>
    </div>
  )
} 