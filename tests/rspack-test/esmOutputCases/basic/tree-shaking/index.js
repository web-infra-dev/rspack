import 'side-effect-free'

// access module so entry is not scope hoisted
console.log.bind(module)
