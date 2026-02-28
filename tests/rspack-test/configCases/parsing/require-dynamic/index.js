const dir = process.env.name

const require1 = require(dir)

const require2 = require('./other.js')

const require3 = require('./foo/' + dir + '.js')

const require4 = require(a + './foo/' + dir + '.js')

const require5 = require(dir ? './foo/' + dir + '.js' : './foo/nested' + dir + 'js')
