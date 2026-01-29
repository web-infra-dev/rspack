module.exports = {
    entry: './index.js',
    mode: 'development',
    stats: {
        assets: true,
        modules: true,
    },
    output: {
        filename: 'bundle.js',
    }
}
