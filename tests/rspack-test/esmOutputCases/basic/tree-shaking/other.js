import 'side-effect-free'

import './other'

// access module so entry is not scope hoisted
console.log.bind(module)
