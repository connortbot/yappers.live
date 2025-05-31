import { useState, useCallback } from 'react'
import type { TeamDraftManager } from '../lib/bindings/TeamDraftManager'
import type { GameMessage } from '../lib/bindings/GameMessage'

export function useTeamDraft() {
  const [teamDraftState, setTeamDraftState] = useState<TeamDraftManager | null>(null)

  const updateTeamDraftState = useCallback((newState: TeamDraftManager) => {
    setTeamDraftState(newState)
  }, [])

  const handleTeamDraftMessage = useCallback((message: GameMessage) => {
    if (message.type === 'TeamDraft') {
      setTeamDraftState(prev => {
        if (!prev) return prev
        
        const updated = { ...prev }
        
        switch (message.msg_type) {
          case 'SetPool':
            updated.round_data = {
              ...updated.round_data,
              pool: message.pool
            }
            break
          case 'SetCompetition':
            updated.round_data = {
              ...updated.round_data,
              competition: message.competition
            }
            break
        }
        
        return updated
      })
    }
  }, [])

  const createPoolMessage = useCallback((pool: string): GameMessage => {
    return {
      type: 'TeamDraft',
      msg_type: 'SetPool',
      pool: pool
    } as any
  }, [])

  const createCompetitionMessage = useCallback((competition: string): GameMessage => {
    return {
      type: 'TeamDraft',
      msg_type: 'SetCompetition',
      competition: competition
    } as any
  }, [])

  const resetTeamDraftState = useCallback(() => {
    setTeamDraftState(null)
  }, [])

  return {
    teamDraftState,
    updateTeamDraftState,
    handleTeamDraftMessage,
    createPoolMessage,
    createCompetitionMessage,
    resetTeamDraftState
  }
} 