'use client'

import { useCallback, useSyncExternalStore } from 'react'

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
let isClientLoaded = false

function getStoredData(): LocalPlayerData {
  if (typeof window === 'undefined') return emptyData
  isClientLoaded = true
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

function getIsLoaded(): boolean {
  return isClientLoaded
}

function subscribe(callback: () => void) {
  window.addEventListener('storage', callback)
  return () => window.removeEventListener('storage', callback)
}

export function useLocalPlayer() {
  const data = useSyncExternalStore(subscribe, getStoredData, () => emptyData)
  // Track if we're on the client side using useSyncExternalStore to avoid useEffect setState
  const isLoaded = useSyncExternalStore(subscribe, getIsLoaded, () => false)

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
