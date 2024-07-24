/* eslint-disable node/no-unsupported-features/es-syntax */
/** eslint-disable **/
import * as path from "path";

// cspell:ignore elopment
const mode: string = "dev" + "elopment";
const config = {
  mode,
  entry: "./main.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "foo.bundle.js",
  },
};

export = config;
