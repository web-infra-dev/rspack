const path = require('path')

module.exports = {
    context: __dirname,
    entry: {
        index: path.resolve(__dirname, 'index.js')
    },
    optimization: {
        moduleIds: 'named',
        minimize: false,
        runtimeChunk: {
            name: 'runtime'
        }
    },
    output: {
        filename: '[name].[contenthash].js'
    }
}