export { readFile as readFileWrapped, readFileSync as readFileSyncWrapped } from 'fs'

console.log.bind(module)
