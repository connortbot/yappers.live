'use client'

import { useRouter } from 'next/navigation'
import { useEffect } from 'react'
import { Button } from '@/components/Button'
import { Screen } from '@/components/Screen'
import { AnimatedBackground } from '@/components/AnimatedBackground'
import { useLocalPlayer } from '@/hooks/useLocalPlayer'
import Image from 'next/image'

export default function Home() {
  const router = useRouter()
  const { gameId, playerId, isLoaded } = useLocalPlayer()

  // Auto-rejoin if player has existing game
  useEffect(() => {
    if (isLoaded && gameId && playerId) {
      router.push(`/game/${gameId}`)
    }
  }, [isLoaded, gameId, playerId, router])

  const handleCreateGame = () => {
    router.push('/lobby/create')
  }

  const handleJoinGame = () => {
    router.push('/lobby/join')
  }

  // Show nothing while checking for existing game
  if (!isLoaded) {
    return (
      <>
        <AnimatedBackground />
        <Screen centered>
          <p className="font-secondary text-pencil">Loading...</p>
        </Screen>
      </>
    )
  }

  return (
    <>
      <AnimatedBackground />
      <Screen centered>
        <div className="flex items-center justify-center gap-4 sm:gap-6 mb-8 sm:mb-12">
          <Image 
            src="/logo-transparent.png" 
            alt="Yappers.live logo" 
            width={112}
            height={112}
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
            onClick={handleCreateGame}
            className="w-full text-lg sm:text-xl md:text-2xl py-3 sm:py-4"
          >
            Create Game
          </Button>
          
          <Button
            variant="primary"
            size="large"
            onClick={handleJoinGame}
            className="w-full text-lg sm:text-xl md:text-2xl py-3 sm:py-4"
          >
            Join Game
          </Button>
        </div>
      </Screen>
    </>
  )
}
