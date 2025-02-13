const { experiments: { SubresourceIntegrityPlugin } } = require("@rspack/core");
const WebpackBeforeBuildPlugin = require("before-build-webpack");
const webpack = require("@rspack/core");
const path = require("path");
const { RunInPuppeteerPlugin, createIntegrityPlugin, createHtmlPlugin, getHtmlPlugin, getDist } = require("../wsi-test-helper");

module.exports = {
  resolve: {
    extensions: [".js", ".jsx"],
  },
  entry: {
    alpha: ["./alpha", "./a"],
    beta: ["./beta", "./b", "./c"],
  },
  output: {
    filename: "MyDll.[name].js",
    library: "[name]_[fullhash]",
    path: getDist(__dirname),
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.join(getDist(__dirname), "[name]-manifest.json"),
      name: "[name]_[fullhash]",
    }),
    new WebpackBeforeBuildPlugin(
      function (_stats, callback) {
        webpack(
          {
            mode: "production",
            entry: {
              index: "./index.js",
            },
            output: {
              path: getDist(__dirname),
              crossOriginLoading: "anonymous",
            },
            plugins: [
              new webpack.DllReferencePlugin({
                context: path.join(__dirname),
                manifest: require(path.join(
                  getDist(__dirname),
                  "alpha-manifest.json"
                )), // eslint-disable-line
              }),
              new webpack.DllReferencePlugin({
                scope: "beta",
                manifest: require(path.join(
                  getDist(__dirname),
                  "beta-manifest.json"
                )), // eslint-disable-line
                extensions: [".js", ".jsx"],
              }),
              createHtmlPlugin(),

              {
                apply: (compiler) => {
                  compiler.hooks.thisCompilation.tap(
                    "wsi-test",
                    (compilation) => {
                      const hooks = getHtmlPlugin().getHooks(compilation);

                      hooks.alterAssetTags.tapPromise(
                        "wsi-test",
                        async (data) => {
                          ["MyDll.alpha.js", "MyDll.beta.js"].forEach((src) => {
                            data.assetTags.scripts.unshift({
                              tagName: "script",
                              voidTag: false,
                              attributes: { defer: true, src },
                            });
                          });
                          return data;
                        }
                      );
                    }
                  );
                },
              },

              createIntegrityPlugin({
                hashFuncNames: ["sha256", "sha384"],
                enabled: true,
              }),

              new RunInPuppeteerPlugin(),
            ],
          },
          function afterEmit(err, stats) {
            if (err || stats.hasErrors() || stats.hasWarnings()) {
              callback(err || new Error(stats.toString({ reason: true })));
            } else {
              callback();
            }
          }
        );
      },
      ["done"]
    ),
  ],
};
