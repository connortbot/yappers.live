'use client'

import { useState, useCallback, useEffect, useSyncExternalStore } from 'react'

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

let cachedData: LocalPlayerData = emptyData
let cachedRaw: string | null = null

function getStoredData(): LocalPlayerData {
  if (typeof window === 'undefined') return emptyData
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored !== cachedRaw) {
      cachedRaw = stored
      cachedData = stored ? JSON.parse(stored) as LocalPlayerData : emptyData
    }
    return cachedData
  } catch {
    return cachedData
  }
}

function subscribe(callback: () => void) {
  window.addEventListener('storage', callback)
  return () => window.removeEventListener('storage', callback)
}

export function useLocalPlayer() {
  const data = useSyncExternalStore(subscribe, getStoredData, () => emptyData)
  const [isLoaded, setIsLoaded] = useState(false)
  
  useEffect(() => {
    setIsLoaded(true)
  }, [])

  const savePlayer = useCallback((playerId: string, gameId: string, username: string) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ playerId, gameId, username }))
      window.dispatchEvent(new Event('storage'))
    } catch {
      // Storage unavailable
    }
  }, [])

  const clearPlayer = useCallback(() => {
    try {
      localStorage.removeItem(STORAGE_KEY)
      window.dispatchEvent(new Event('storage'))
    } catch {
      // Storage unavailable
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
