/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry: "./index",
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.done.tap("TestPlugin", stats => {
                    const erros = stats.compilation.errors;
                    expect(erros.length).toBe(1);
                    expect(erros[0].name).toBe("ModuleBuildError");
                    expect(erros[0].message).toContain("CustomError: Custom mesasge");

                    expect(erros[0].error.name).toBe("CustomError");
                    expect(erros[0].error.message).toContain("Custom mesasge");
                });
            }
        }
    ],
    module: {
        rules: [
            {
                test: /\.js$/,
                use: {
                    loader: require.resolve("./loader"),
                }
            }
        ]
    }
};
