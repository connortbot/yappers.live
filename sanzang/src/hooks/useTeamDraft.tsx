import { useState, useCallback } from 'react'
import type { TeamDraftManager } from '../lib/bindings/TeamDraftManager'
import type { GameMessage } from '../lib/bindings/GameMessage'
import { TeamDraftMessage } from '../lib/bindings/TeamDraftMessage'

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
          case 'StartDraft':
            updated.phase = 'Drafting'
            updated.round_data = {
              ...updated.round_data,
              starting_drafter_id: message.starting_drafter_id,
              current_drafter_id: message.starting_drafter_id,
              player_to_picks: { ...(updated.round_data.player_to_picks || {}) }
            }
            break
          case 'DraftPick':
            if (updated.round_data) {
              const newPlayerToPicks = { ...updated.round_data.player_to_picks }
              if (!newPlayerToPicks[message.drafter_id]) {
                newPlayerToPicks[message.drafter_id] = []
              }
              newPlayerToPicks[message.drafter_id]!.push(message.pick)
              
              updated.round_data = {
                ...updated.round_data,
                player_to_picks: newPlayerToPicks
              }
            }
            break
          case 'NextDrafter':
            updated.round_data.current_drafter_id = message.drafter_id
            break
          case 'AwardingPhase':
            updated.phase = 'Awarding'
            break
          case 'AwardPoint':
            updated.player_points[message.player_id] = (updated.player_points[message.player_id] ?? 0) + 1
            break
          case 'CompleteGame':
            updated.phase = 'Complete'
            updated.player_points = message.player_points
            break
          case 'NextRound':
            updated.phase = 'YapperChoosing'
            updated.yapper_id = message.yapper_id
            updated.yapper_index = message.yapper_index
            updated.round_data = {
              ...updated.round_data,
              round: message.round,
              team_size: message.team_size,
              player_to_picks: {},
              starting_drafter_id: "",
              current_drafter_id: "",
              pool: "",
              competition: "",
            }
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
    } as GameMessage & TeamDraftMessage
  }, [])

  const createCompetitionMessage = useCallback((competition: string): GameMessage => {
    return {
      type: 'TeamDraft',
      msg_type: 'SetCompetition',
      competition: competition
    } as GameMessage & TeamDraftMessage
  }, [])

  const createStartDraftMessage = useCallback((starting_drafter_id: string): GameMessage => {
    return {
      type: 'TeamDraft',
      msg_type: 'StartDraft',
      starting_drafter_id: starting_drafter_id
    } as GameMessage & TeamDraftMessage
  }, [])

  const createAwardPointMessage = useCallback((player_id: string): GameMessage => {
    return {
      type: 'TeamDraft',
      msg_type: 'AwardPoint',
      player_id: player_id
    } as GameMessage & TeamDraftMessage
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
    createStartDraftMessage,
    createAwardPointMessage,
    resetTeamDraftState
  }
} 