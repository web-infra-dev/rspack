/** @type {import("@rspack/core").Configuration} */
module.exports = {
    devtool: "source-map",
    module: {
        rules: [
            {
                test: /\.(jsx?|tsx?)$/,
                use: [
                    {
                        loader: "builtin:swc-loader"
                    }
                ]
            }
        ]
    },
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.afterEmit.tap("PLUGIN", compilation => {
                    const sourceMap = JSON.parse(compilation.assets["bundle0.js.map"].source());
                    expect(sourceMap.sourcesContent).toContain(
                        'export default name => console.log(`hello, ${name}!`);'
                    );
                    expect(sourceMap.sources).toEqual(expect.arrayContaining([
                        'webpack:///./node_modules/foo/index.js',
                        'webpack:///./index.js'
                    ]));
                });
            }
        }
    ]
};
