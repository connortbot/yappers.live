import { useNavigate } from 'react-router'
import { Button } from './components/Button'
import { Screen } from './components/Screen'
import logoTransparent from './assets/logo-transparent.png'

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
      <div className="flex items-center justify-center gap-4 sm:gap-6 mb-8 sm:mb-12">
        <img 
          src={logoTransparent} 
          alt="Yappers.live logo" 
          className="w-12 h-12 xs:w-16 xs:h-16 sm:w-20 sm:h-20 md:w-24 md:h-24 lg:w-28 lg:h-28"
        />
        <h1 className="text-4xl xs:text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-bold font-primary text-center leading-tight">
          yappers.live
        </h1>
      </div>
      
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