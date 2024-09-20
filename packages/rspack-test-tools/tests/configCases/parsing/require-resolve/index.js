const dir = process.env.DIR_READ_FROM_RUNTIME

const resolve1 = require.resolve(dir)

const resolve2 = require.resolve('./other.js')

const resolve3 = require.resolve('./foo/' + dir + '.js')

const resolve4 = require.resolve(process.env.RANDOM ? './foo/' + dir + '.js' : './bar/' + dir + 'js')


// Can't handle, `require` will turn into expression
// const resolve5 = require.resolve
// resolve5('./other.js')

// Can't handle, `require` will turn into `undefined`
// const __require = require
// const resolve6 = __require.resolve('./other.js')

