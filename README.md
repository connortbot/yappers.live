# yappers.live

By mickey mouse
**Social games for the dinner table, no app required.**

Play now at [yappers.live](https://yappers.live).

## Games:
- Spyfall
- Cross Clues (not done)
- Werewolf (coming soon)
- Would You Rather (planned)
- Hot Takes (brainstorming phase)

## Tech Stack
- Bun
- Next + tailwind
- Valkey
- Vibes

## Local Dev

```bash
cp .env.example .env

# Valkey
docker compose -f docker-compose-local.yml up -d

# Web
bun run dev

# Visit http://localhost:3000
```

## Contributing
PRs welcome! The codebase is intentionally simple.

## License
MIT
