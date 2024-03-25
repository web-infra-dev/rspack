import { fileURLToPath } from "url";
import path from "path";

export default {
  entry: "./a.js",
  output: {
    path: path.resolve(path.dirname(fileURLToPath(import.meta.url)), "dist"),
    filename: "a.bundle.js",
  },
};
