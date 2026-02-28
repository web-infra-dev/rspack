module.exports = {
    entry: './index.js',
    mode: 'development',
    stats: {
        assets: true,
        modules: true,
    },
    optimization: { minimize: false },
    output: {
        filename: 'bundle.js',
    }
}
