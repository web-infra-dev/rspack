import path from "path";
import { fileURLToPath } from "url";

export default {
  entry: "./a.js",
  output: {
    path: path.resolve(path.dirname(fileURLToPath(import.meta.url)), "dist"),
    filename: "a.bundle.js",
  },
};
