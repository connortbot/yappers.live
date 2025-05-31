import { useNavigate } from 'react-router'
import { Button } from './components/Button'
import { Screen } from './components/Screen'

export default function Home() {
  const navigate = useNavigate()

  const handleCreateGame = () => {
    navigate('/lobby/create')
  }

  const handleJoinGame = () => {
    navigate('/lobby/join')
  }

  return (
    <Screen centered>
      <h1 className="text-4xl xs:text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-bold mb-8 sm:mb-12 font-primary text-center leading-tight">
        yappers.live
      </h1>
      
      <div className="flex flex-col gap-4 sm:gap-6 w-full max-w-xs sm:max-w-sm">
        <Button
          variant="primary"
          size="large"
          onMouseUp={handleCreateGame}
          className="w-full text-lg sm:text-xl md:text-2xl py-3 sm:py-4"
        >
          Create Game
        </Button>
        
        <Button
          variant="primary"
          size="large"
          onMouseUp={handleJoinGame}
          className="w-full text-lg sm:text-xl md:text-2xl py-3 sm:py-4"
        >
          Join Game
        </Button>
      </div>
    </Screen>
  )
} 