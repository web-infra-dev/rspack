module.exports = {
    entry: './index.js',
    mode: 'development',
    devtool: 'eval',
    stats: {
        assets: true,
        modules: true,
    },
    output: {
        filename: 'bundle.js',
    }
}
