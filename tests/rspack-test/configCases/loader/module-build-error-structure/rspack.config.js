/** @type {import("@rspack/core").Configuration} */
module.exports = {
    entry: "./index.js",
    module: {
        rules: [
            {
                test: /\.js$/i,
                use: [{ loader: "./loader" }],
            }
        ]
    },
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.done.tap("PLUGIN", stats => {
                    const { errors } = stats.compilation;
                    expect(errors).toHaveLength(1);

                    const error = errors[0];
                    expect(error).toMatchObject({
                        name: "ModuleBuildError",
                        error: {
                            name: "NextFontError",
                            message: "Cannot be used within pages/_document.js"
                        }
                    });
                });
            }
        }
    ]
};
