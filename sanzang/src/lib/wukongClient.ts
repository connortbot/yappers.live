import createClient from "openapi-fetch";
import type { paths } from "./wukong";

export const $WUKONG = createClient<paths>({ baseUrl: import.meta.env.VITE_APP_WUKONG_BASE_URL });

export const createWukongWebSocket = (path: string) => {
    return new WebSocket(`${import.meta.env.VITE_APP_WUKONG_WEBSOCKET_BASE_URL}${path}`);
}