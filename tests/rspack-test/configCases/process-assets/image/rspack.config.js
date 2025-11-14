/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
    context: __dirname,
    module: {
        rules: [
            {
                test: /\.png$/,
                type: "asset"
            },
        ]
    },
    plugins: [
        compiler => {
            compiler.hooks.compilation.tap("PLUGIN", compilation => {
                compilation.hooks.processAssets.tap("PLUGIN", assets => {
                    for (const name in assets) {
                        if (name.endsWith("png")) {
                            expect(assets[name].source()).toBeInstanceOf(Buffer);
                        }
                    }
                })
            })
        }
    ]
};
