/* eslint-disable */

import path from "path";
import { fileURLToPath } from "url";

export default {
  mode: "development",
  entry: "./main.js",
  output: {
    path: path.resolve(path.dirname(fileURLToPath(import.meta.url)), "dist"),
    filename: "foo.bundle.js",
  },
};
