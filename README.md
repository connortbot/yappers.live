# yappers.live
**Social games for the dinner table, no app required.**

Play now at [yappers.live](https://yappers.live).

## Current Game: Yappers
A Spyfall-style party game for 3+ players:
- One player is secretly the **Spy**
- Everyone else knows the **Thing** (a player's name)
- Discuss and figure out who the Spy is!

## Tech Stack
- **Next.js 16** (App Router) - frontend + backend
- **Tailwind CSS 4** - styling
- **Redis** - game state storage (Valkey locally, Redis Cloud in prod)
- **Vercel** - hosting

## Development

```bash
# Start Valkey locally
docker compose -f docker-compose-local.yml up -d

# Start dev server
bun run dev

# Visit http://localhost:3000
```

## Environment Variables

```bash
# .env.local
REDIS_URL=redis://localhost:6379
```

## Contributing
PRs welcome! The codebase is intentionally simple.
