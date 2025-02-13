const { createIntegrityPlugin, createHtmlPlugin, getDist } = require("../wsi-test-helper");
const HtmlWebpackExternalsPlugin = require("html-webpack-externals-plugin");
const expect = require("expect");
const htmlparser2 = require("htmlparser2");
const { readFileSync } = require("fs");
const { selectAll } = require("css-select");
const { join } = require("path");
module.exports = {
  mode: "production",
  entry: "./index.js",
  output: {
    filename: "bundle.js",
    publicPath: "/",
    crossOriginLoading: "anonymous",
    path: getDist(__dirname),
  },
  plugins: [
    createHtmlPlugin({
      inject: "body",
    }),
    new HtmlWebpackExternalsPlugin({
      externals: [
        {
          module: "jquery",
          entry: {
            path: "https://code.jquery.com/jquery-3.2.1.js",
            attributes: {
              integrity: "sha256-DZAnKJ/6XZ9si04Hgrsxu/8s717jcIzLy3oi35EouyE=",
              crossorigin: "anonymous",
            },
          },
          global: "jQuery",
        },
      ],
    }),
    createIntegrityPlugin({
      hashFuncNames: ["sha256", "sha384"],
    }),
    {
      apply: (compiler) => {
        compiler.hooks.done.tapPromise("wsi-test", async (stats) => {
          expect(stats.compilation.warnings.length).toEqual(0);

          const dom = htmlparser2.parseDocument(
            readFileSync(join(getDist(__dirname), "index.html"), "utf-8")
          );

          const scripts = selectAll("script", dom);
          expect(scripts.length).toEqual(2);
          for (let i = 0; i < scripts.length; i += 1) {
            expect(scripts[0].attribs.crossorigin).toEqual("anonymous");
            expect(scripts[0].attribs.integrity).toMatch(/^sha/);
          }
        });
      },
    },
  ],
};
