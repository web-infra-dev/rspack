// eslint-disable-next-line node/no-unsupported-features/es-syntax
import * as path from "path";

const config = {
  mode: "development",
  entry: "./main.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "foo.bundle.js",
  },
};

export = config;
