class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.done.tap("PLUGIN", stats => {
            const json = stats.toJson();
            expect(json.warnings).toHaveLength(0);
        });
    }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    module: {
        parser: {
            "css/auto": {
                namedExports: true
            }
        },
        rules: [
            {
                test: /\.css$/,
                use: [
                    {
                        loader: "builtin:lightningcss-loader",
                        /** @type {import("@rspack/core").LightningcssLoaderOptions} */
                        options: {
                            unusedSymbols: ["unused"],
                            targets: "> 0.2%"
                        }
                    }
                ],
                type: "css/auto"
            }
        ]
    },
    experiments: {
        css: true
    },
    plugins: [
        new Plugin()
    ]
};
