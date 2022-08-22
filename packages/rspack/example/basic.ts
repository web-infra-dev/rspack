import path from "path";
import postcssLoader from "rspack-plugin-postcss";
import { Rspack } from "../src";

const rspack = new Rspack({
  entry: {
    main: path.resolve(__dirname, "../../../examples/react/src/index.js"),
  },
  context: path.resolve(__dirname, "../../../examples/react"),
  plugins: ["html"],
  module: {
    rules: [
      {
        test: ".css",
        uses: [postcssLoader],
      },
    ],
  },
});

async function main() {
  const stats = await rspack.build();
  console.log(stats);
  // assert(stats.assets.length > 0)
}

main();
