const fs = require('fs');
const words = fs
  .readFileSync('./project-words.txt', { encoding: 'utf-8' })
  .split('\n')
  .filter(Boolean);

const sortedWords = words.sort((a, b) => a.localeCompare(b));

fs.writeFileSync('./project-words.txt', sortedWords.join('\n') + '\n');
