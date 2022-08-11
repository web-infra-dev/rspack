import path from "path"
import { Rspack } from ".."
import assert from 'assert';

const rspack = new Rspack({
  entry: {
    main: path.resolve(__dirname, "../../../examples/react/src/index.js"),
  },
  context: path.resolve(__dirname, "../../../examples/react")
})

async function main() {
  const stats = await rspack.build()
  assert(stats.assets.length > 0)
}

main()