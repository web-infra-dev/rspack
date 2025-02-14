const { createIntegrityPlugin, createHtmlPlugin, getDist } = require("../wsi-test-helper");
const { CssExtractRspackPlugin } = require("@rspack/core");
const expect = require("expect");
const htmlparser2 = require("htmlparser2");
const { readFileSync } = require("fs");
const { selectAll } = require("css-select");
const { join } = require("path");

module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename: "subdir/bundle.js",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [CssExtractRspackPlugin.loader, "css-loader"],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: "subdir/styles.css",
      chunkFilename: "[id].css",
    }),
    createHtmlPlugin({
      hash: true,
      inject: false,
      filename: "admin.html",
      template: process.env.HTML_PLUGIN === "rspack" ? "index.rspack.ejs" : "index.ejs",
    }),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"]
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tap("wsi-test", (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);
          expect(stats.compilation.errors.length).toEqual(0);

          const jsIntegrity = stats
            .toJson()
            .assets.find(
              (asset) => asset.name === "subdir/bundle.js"
            ).integrity;
          expect(jsIntegrity).toMatch(/^sha/);

          const cssIntegrity = stats
            .toJson()
            .assets.find(
              (asset) => asset.name === "subdir/styles.css"
            ).integrity;
          expect(cssIntegrity).toMatch(/^sha/);

          const dom = htmlparser2.parseDocument(
            readFileSync(join(getDist(__dirname), "admin.html"), "utf-8")
          );

          const scripts = selectAll("script", dom);
          expect(scripts.length).toEqual(1);
          expect(scripts[0].attribs.crossorigin).toEqual("anonymous");
          expect(scripts[0].attribs.integrity).toEqual(jsIntegrity);

          const links = selectAll("link", dom);
          expect(links.length).toEqual(1);
          expect(links[0].attribs.crossorigin).toEqual("anonymous");
          expect(links[0].attribs.integrity).toEqual(cssIntegrity);
        });
      },
    },
  ],
};
