const fs = require("node:fs");
const path = require("path");
const { rspack } = require("@rspack/core");

const libraryPath = path.resolve(process.argv[2]);
const outputPath = path.join(libraryPath, "consumer");
const compiler = rspack({
  context: __dirname,
  mode: "production",
  target: "node",
  entry: "./consumer.js",
  devtool: false,
  output: {
    path: outputPath,
    filename: "consumer.js",
  },
  optimization: {
    minimize: true,
  },
  resolve: {
    alias: {
      "entry-namespace-library$": path.join(libraryPath, "main.mjs"),
    },
  },
});

compiler.run((error, stats) => {
  compiler.close(() => {});
  if (error) {
    throw error;
  }
  if (!stats || stats.hasErrors()) {
    throw new Error(
      stats?.toString({ all: false, errors: true }) ?? "Missing consumer stats",
    );
  }

  const source = fs.readFileSync(path.join(outputPath, "consumer.js"), "utf-8");
  if (!source.includes("KEEP_FOO_SENTINEL")) {
    throw new Error("The used namespace export was removed from the consumer");
  }
  if (source.includes("DROP_BAR_SENTINEL")) {
    throw new Error("The unused namespace export was retained in the consumer");
  }
});
