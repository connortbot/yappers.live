import { useCallback } from 'react';
import { components } from '../lib/wukong';
import { $WUKONG } from '../lib/wukongClient';

export function useGameAPI() {
    const createGame = useCallback(async (params: {
        username: string;
    }) => {
        const { data, error } = await $WUKONG.POST('/game/create', {
            body: params
        });
        return {
            data: data as components["schemas"]["CreateGameResponse"] | null,
            error: error as components["schemas"]["ErrorResponse"] | null
        }
    }, []);

    const joinGame = useCallback(async (params: {
        game_code: string;
        username: string;
    }) => {
        const { data, error } = await $WUKONG.POST('/game/join', {
            body: params
        });
        return {
            data: data as components["schemas"]["JoinGameResponse"] | null,
            error: error as components["schemas"]["ErrorResponse"] | null
        }
    }, []);
    
    return { createGame, joinGame };
}