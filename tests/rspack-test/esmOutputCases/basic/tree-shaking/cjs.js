import './use-other'

// use export readFile
console.log.bind(require('fs').readFile)
