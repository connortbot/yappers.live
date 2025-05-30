import { useCallback } from 'react';
import { components } from '../lib/wukong';
import { $WUKONG } from '../lib/wukongClient';

export function useAdminAPI() {
    const listGames = useCallback(async (adminPassword: string) => {
        const { data, error } = await $WUKONG.GET('/admin/games', {
            headers: {
                'WukongAdminPassword': adminPassword
            }
        });
        return {
            data: data as components["schemas"]["GamesListResponse"] | null,
            error: error as components["schemas"]["ErrorResponse"] | null
        }
    }, []);

    const getGame = useCallback(async (gameId: string, adminPassword: string) => {
        const { data, error } = await $WUKONG.GET('/admin/game', {
            params: {
                query: {
                    id: gameId
                }
            },
            headers: {
                'WukongAdminPassword': adminPassword
            }
        });
        return {
            data: data as components["schemas"]["GameDetailsResponse"] | null,
            error: error as components["schemas"]["ErrorResponse"] | null
        }
    }, []);
    
    return { listGames, getGame };
} 