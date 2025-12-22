const path = require("path");
const fs = require("fs");
const os = require("os");
/**
 * @type {import('@rspack/core').Configuration}
 */
const config = {
  mode: "development",
  target: "web",
  devtool: false,
  entry: {
    app: "./src/app",
    app2: "./src/app2",
  },
  node: false,
  plugins: [{
    apply(compiler) {
      compiler.hooks.done.tapAsync('TestPlugin', ({ compilation }, callback) => {
        const chunks = [];

        Array.from(compilation.entrypoints.values()).forEach((entrypoint) => {
          entrypoint.chunks.forEach(chunk => {
            // Simulate some processing on each chunk
            chunks.push(`${chunk.name}, ${Array.from(chunk.files).join(', ')}, ${Array.from(chunk.auxiliaryFiles).join(', ')}`);
          });
        });

        fs.writeFileSync(path.join(compiler.outputPath, 'chunks-summary.txt'), chunks.join(os.EOL), 'utf-8');
        callback()
      });
    }
  }],
  optimization: {
    runtimeChunk: true,
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: {
      name(module, chunks) {
        return chunks.map((item) => item.name).join('~');
      },
      minSize: 0,
      chunks: 'all',
    },
  },
  externals: {
    underscore: {
      root: "fs",
    },
    jquery: {
      root: "fs",
    }
  },
  output: {
    filename: "[name].js",
  },
};

module.exports = config;
