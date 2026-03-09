import 'side-effect-free'

import './cjs'

// access module so entry is not scope hoisted
console.log.bind(module)
