const path = require('path');

const DisplayStatsPlugin = (compiler) => {
    compiler.hooks.done.tap("DisplayStatsPlugin", stats => {
        const statsJSON = stats.toJson();
        for (const chunk of statsJSON.chunks) {
            for (const module of chunk.modules) {
                console.log(module.id, module.reasons)
            }
        }
    })
}

module.exports = {
    stats: {reasons: true},
    entry: './index.js',
    mode: 'development',
    target: 'node',
    output: {
        filename: 'main.js',
        path: path.resolve(__dirname, 'dist'),
    },
    plugins: [DisplayStatsPlugin]
};