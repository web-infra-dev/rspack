import path from "path"
import { Rspack } from ".."


const rspack = new Rspack({
  entries: {
    main: path.resolve(__dirname, "../../../examples/react/src/index.js"),
  },
  root: path.resolve(__dirname, "../../../examples/react")
})

async function main() {
  const stats = await rspack.build()
  console.log(stats);
}

main()