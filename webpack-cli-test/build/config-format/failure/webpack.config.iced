path = require 'path'

config =
    mode: 'development'
    entry: './main.js'
    output:
        path: path.resolve(__dirname, 'dist')
        filename: 'foo.bundle.js'

module.exports = config;
