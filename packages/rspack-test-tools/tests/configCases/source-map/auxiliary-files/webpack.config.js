/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "production",
    devtool: 'source-map',
    plugins: [
        compiler => {
            const { Compilation } = compiler.webpack;
            compiler.hooks.thisCompilation.tap("test case", compilation => {
                compilation.hooks.processAssets.tap({
                    name: "test case",
                    stage: Compilation.PROCESS_ASSETS_STAGE_REPORT
                }, () => {
                    expect(Array.from(compilation.chunks[0].auxiliaryFiles)).toEqual(["bundle0.js.map"]);
                });
            });
        }
    ]
};
