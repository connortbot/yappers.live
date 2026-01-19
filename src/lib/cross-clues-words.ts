// Word pool for Cross Clues game
// These words should be simple nouns/concepts that can combine in interesting ways

export const CROSS_CLUES_WORDS: string[] = [
  // Animals
  'dog', 'cat', 'bird', 'fish', 'bear', 'lion', 'elephant', 'horse', 'monkey', 'snake',
  'rabbit', 'tiger', 'wolf', 'fox', 'mouse', 'pig', 'cow', 'sheep', 'chicken', 'duck',

  // Nature
  'tree', 'flower', 'mountain', 'river', 'ocean', 'sun', 'moon', 'star', 'cloud', 'rain',
  'snow', 'fire', 'water', 'earth', 'wind', 'forest', 'desert', 'island', 'beach', 'garden',

  // Objects
  'book', 'phone', 'car', 'house', 'chair', 'table', 'door', 'window', 'key', 'clock',
  'mirror', 'lamp', 'bed', 'couch', 'computer', 'camera', 'guitar', 'piano', 'ball', 'ring',

  // Food & Drink
  'apple', 'bread', 'cheese', 'coffee', 'pizza', 'cake', 'candy', 'ice cream', 'wine', 'beer',
  'chocolate', 'banana', 'orange', 'egg', 'milk', 'honey', 'salt', 'pepper', 'rice', 'pasta',

  // Places
  'school', 'hospital', 'bank', 'church', 'museum', 'theater', 'restaurant', 'airport', 'hotel', 'park',
  'library', 'prison', 'castle', 'farm', 'factory', 'office', 'stadium', 'mall', 'zoo', 'circus',

  // Concepts
  'love', 'money', 'time', 'music', 'art', 'war', 'peace', 'dream', 'fear', 'hope',
  'family', 'friend', 'king', 'queen', 'baby', 'doctor', 'teacher', 'police', 'soldier', 'ghost',

  // Activities
  'dance', 'sport', 'game', 'party', 'wedding', 'birthday', 'travel', 'work', 'sleep', 'cook',

  // Colors & Qualities
  'red', 'blue', 'green', 'gold', 'silver', 'black', 'white', 'hot', 'cold', 'fast',

  // Body
  'heart', 'brain', 'hand', 'eye', 'mouth', 'hair', 'blood', 'bone', 'skin', 'muscle',

  // Technology & Modern
  'robot', 'rocket', 'internet', 'video', 'magic', 'science', 'space', 'planet', 'alien', 'zombie',

  // Misc
  'night', 'day', 'summer', 'winter', 'spring', 'fall', 'christmas', 'halloween', 'pirate', 'ninja'
]

// Shuffle array using Fisher-Yates algorithm
function shuffleArray<T>(array: T[]): T[] {
  const shuffled = [...array]
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
  }
  return shuffled
}

// Get random words for a Cross Clues game (10 unique words: 5 for rows, 5 for columns)
export function getRandomWords(): { rowWords: string[], colWords: string[] } {
  const shuffled = shuffleArray(CROSS_CLUES_WORDS)
  return {
    rowWords: shuffled.slice(0, 5),
    colWords: shuffled.slice(5, 10)
  }
}

// Generate all 25 coordinate cards
export function generateCards(): { id: string, coordinate: string, assignedTo: string | null }[] {
  const cards: { id: string, coordinate: string, assignedTo: string | null }[] = []
  const columns = ['A', 'B', 'C', 'D', 'E']
  const rows = ['1', '2', '3', '4', '5']

  for (const col of columns) {
    for (const row of rows) {
      const coordinate = `${col}${row}`
      cards.push({
        id: `card-${coordinate}`,
        coordinate,
        assignedTo: null
      })
    }
  }

  // Shuffle the cards
  return shuffleArray(cards)
}
