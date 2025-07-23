import path from 'path'

const dir = process.env.DIR_READ_FROM_RUNTIME

const resolve1 = require.resolve(dir)

const resolve2 = require.resolve('./other.js')

const resolve3 = require.resolve('./foo/' + dir + '.js')

const resolve4 = require.resolve(process.env.RANDOM ? './foo/' + dir + '.js' : './bar/' + dir + 'js')

const resolve5 = require.resolve(path.resolve(__dirname, './other.js'))

const resolve6 = require.resolve('./a', { paths: [ cwd, path.resolve(cwd, 'node_modules') ] })

// Can't handle, `require` will turn into expression
// const resolve7 = require.resolve
// resolve7('./other.js')

// Can't handle, `require` will turn into `undefined`
// const __require = require
// const resolve8 = __require.resolve('./other.js')

