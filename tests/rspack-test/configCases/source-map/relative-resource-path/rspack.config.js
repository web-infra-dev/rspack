const path = require("path");
const fs = require("fs");

function normalizeToUrlStyle(s) {
    // 1) Convert Windows backslashes to forward slashes
    const withForward = s.replace(/\\/g, "/");
    // 2) POSIX-normalize to collapse ".." / "." segments
    return path.posix.normalize(withForward);
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
    mode: "development",
    devtool: "source-map",
    entry: "./src/index.js",
    output: {
        filename: 'static/js/[name].js',
        devtoolModuleFilenameTemplate: '[relative-resource-path]',
    },
    plugins: [
        {
            apply(compiler) {
                compiler.hooks.done.tap("PLUGIN", stats => {
                    const outputPath = stats.compilation.getPath(compiler.outputPath, {});
                    const sourceMapPath = path.join(outputPath, "static/js/main.js.map");
                    const sourceMapJSON = fs.readFileSync(sourceMapPath, "utf-8");
                    const sourceMap = JSON.parse(sourceMapJSON);

                    const realSources = sourceMap.sources.filter(s => !s.startsWith("webpack/"));

                    realSources.forEach(s => {
                        expect(path.isAbsolute(s)).toBe(false);
                        // sources in a source map should be relative/URL-style (not absolute filesystem paths)
                        expect(normalizeToUrlStyle(s)).toBe(s);
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
