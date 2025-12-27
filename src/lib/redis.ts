import Redis from 'ioredis'

const getRedisClient = () => {
  const redisUrl = process.env.REDIS_URL || 'redis://localhost:6379'
  return new Redis(redisUrl)
}

// Singleton pattern for Redis client
let redisClient: Redis | null = null

export const redis = () => {
  if (!redisClient) {
    redisClient = getRedisClient()
  }
  return redisClient
}
