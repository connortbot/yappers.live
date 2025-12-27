'use client'

import { useState, useCallback, useSyncExternalStore } from 'react'

interface LocalPlayerData {
  playerId: string | null
  gameId: string | null
  username: string | null
}

const STORAGE_KEY = 'yappers_player'

const emptyData: LocalPlayerData = {
  playerId: null,
  gameId: null,
  username: null,
}

function getStoredData(): LocalPlayerData {
  if (typeof window === 'undefined') return emptyData
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      return JSON.parse(stored) as LocalPlayerData
    }
  } catch (e) {
    console.error('Failed to load player data from localStorage:', e)
  }
  return emptyData
}

function subscribe(callback: () => void) {
  window.addEventListener('storage', callback)
  return () => window.removeEventListener('storage', callback)
}

export function useLocalPlayer() {
  const data = useSyncExternalStore(
    subscribe,
    getStoredData,
    () => emptyData // Server snapshot
  )
  
  const [isLoaded, setIsLoaded] = useState(false)
  
  // Mark as loaded after first render
  if (typeof window !== 'undefined' && !isLoaded) {
    setIsLoaded(true)
  }

  // Save player data
  const savePlayer = useCallback((playerId: string, gameId: string, username: string) => {
    const newData: LocalPlayerData = { playerId, gameId, username }
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(newData))
      // Trigger storage event for other tabs
      window.dispatchEvent(new Event('storage'))
    } catch (e) {
      console.error('Failed to save player data to localStorage:', e)
    }
  }, [])

  // Clear player data
  const clearPlayer = useCallback(() => {
    try {
      localStorage.removeItem(STORAGE_KEY)
      window.dispatchEvent(new Event('storage'))
    } catch (e) {
      console.error('Failed to clear player data from localStorage:', e)
    }
  }, [])

  return {
    playerId: data.playerId,
    gameId: data.gameId,
    username: data.username,
    isLoaded,
    savePlayer,
    clearPlayer,
  }
}
