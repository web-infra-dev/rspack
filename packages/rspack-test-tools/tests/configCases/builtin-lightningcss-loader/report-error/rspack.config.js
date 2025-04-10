class Plugin {
    /**
     * @param {import("@rspack/core").Compiler} compiler
     */
    apply(compiler) {
        compiler.hooks.done.tap("PLUGIN", stats => {
            const json = stats.toJson();
            expect(json.errors).toHaveLength(1);
            expect(json.errors[0].message).toMatch(/Unexpected end of input at/);
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
