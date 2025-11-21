require('./other.js')

const resolve1 = require.resolve
resolve1('./other.js')

const lazyFn = (module, requireFn) => {}
lazyFn('./other.js', require)


