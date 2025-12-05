const path = require("path");
const fs = require("fs");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "development",
    devtool: "source-map",
    entry: "./src/index.js",
    output: {
        devtoolModuleFilenameTemplate: '[relative-resource-path]',
    },
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.done.tap("PLUGIN", stats => {
                    const outputPath = stats.compilation.getPath(compiler.outputPath, {});
                    const sourceMapPath = path.join(outputPath, "bundle0.js.map");
                    const sourceMapJSON = fs.readFileSync(sourceMapPath, "utf-8");
                    const sourceMap = JSON.parse(sourceMapJSON);

                    const realSources = sourceMap.sources.filter(s => !s.startsWith("webpack://"));

                    realSources.forEach(s => {
                        expect(path.isAbsolute(s)).toBe(false);
                    });

                    const mapDir = path.dirname(sourceMapPath);
                    const resolved = realSources.map(s => path.resolve(mapDir, s)).sort();
                    const expectedFiles = [
                        path.resolve(__dirname, "src/index.js"),
                        path.resolve(__dirname, "src/button/index.js")
                    ].sort();
                    expect(resolved).toEqual(expectedFiles);
                })
            }
        }
    ]
};
