const dir = process.env.name

import('./other.js')

import('./' + dir + '/other.js')
import(dir)


