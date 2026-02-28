const path = require('node:path');
const { rspack, experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');

const swcLoaderRule = {
    test: /\.jsx?$/,
    use: [
        {
            loader: 'builtin:swc-loader',
            options: {
                jsc: {
                    parser: {
                        syntax: 'ecmascript',
                        jsx: true,
                    },
                    transform: {
                        react: {
                            runtime: 'automatic',
                        },
                    },
                },
                rspackExperiments: {
                    reactServerComponents: true,
                },
            },
        },
    ],
};

const cssRule = {
    test: /\.css$/,
    type: 'css/auto',
};

module.exports = [
    {
        name: 'server',
        mode: 'production',
        target: 'node',
        entry: {
            main: {
                import: ssrEntry,
            },
        },
        resolve: {
            extensions: ['...', '.ts', '.tsx', '.jsx'],
        },
        module: {
            rules: [
                cssRule,
                swcLoaderRule,
                {
                    resource: ssrEntry,
                    layer: Layers.ssr,
                },
                {
                    resource: rscEntry,
                    layer: Layers.rsc,
                    resolve: {
                        conditionNames: ['react-server', '...'],
                    },
                },
                {
                    issuerLayer: Layers.rsc,
                    resolve: {
                        conditionNames: ['react-server', '...'],
                    },
                },
            ],
        },
        plugins: [
            new ServerPlugin(),
            new rspack.DefinePlugin({
                TODOS_PATH: JSON.stringify(path.join(__dirname, 'src/Todos.js')),
            }),
        ],
        optimization: {
            moduleIds: 'named',
            concatenateModules: true,
        },
        output: {
            filename: '[name].js',
        },
    },
    {
        name: 'client',
        mode: 'production',
        target: 'node',
        entry: {
            main: {
                import: './src/framework/entry.client.js',
            },
        },
        resolve: {
            extensions: ['...', '.ts', '.tsx', '.jsx'],
        },
        module: {
            rules: [
                cssRule,
                swcLoaderRule
            ],
        },
        plugins: [new ClientPlugin()],
        optimization: {
            moduleIds: 'named',
            concatenateModules: true,
        },
        output: {
            filename: 'static/[name].js',
            library: {
                type: 'commonjs',
            },
        },
    },
];
