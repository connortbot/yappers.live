# yappers.live
**Social games for the dinner table, no app required.**

Play now at [yappers.live](https://yappers.live).

## Status
This was a side project of mine to learn Rust, but it's also just a cheap and easy thing to host and play with friends.
I am *actively using it*, but not necessarily *actively maintaining it*.

Some games on the roadmap:
- Mind Match (couples Q&A)
- Wavelength
- Two Truths and a Lie
- Spyfall

## Contributing
I'll actively look at PRs, in case someone wants to add a game, fix a bug, wtv.

Tech stack:
Frontend
- Vite + React + Typescript
- Hosted on Vercel

Backend:
- Rust monolith
- Dockerized, -> Render hosting
- Valkey for game state and pubsub

```bash
# Sanzang (frontend)
npm run dev

# Wukong (backend)
cargo watch -x run

# Run Valkey locally
docker-compose -f docker-compose-local.yml up -d
valkey-cli
MONITOR
```
