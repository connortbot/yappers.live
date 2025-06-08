# yappers.live
*Real fun, real connections. Just pull up a website.*



# Dev
Tech stack:
Frontend
- Vite + React + Typescript
- Hosted on Vercel

Backend:
- Rust monolith
- Dockerized, -> Render hosting
(later ECS?)

Postgres + Auth:
- Supabase

Cache:
I wanna use Valkey but like, I really don't need it.


### Local Dev
- sanzang: `npm run dev`
- wukong: `cargo watch -x run`
valkey: 
- `docker-compose -f docker-compose-local.yml up -d`
- `valkey-cli`, `MONITOR`

### Repo Complexity
Score: `4`