import createClient from "openapi-fetch";
import type { paths } from "./wukong";

const $WUKONG = createClient<paths>({ baseUrl: import.meta.env.VITE_APP_WUKONG_API_URL });
export default $WUKONG;