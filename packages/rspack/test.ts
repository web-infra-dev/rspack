import path from "path"

import { Rspack } from "."

const options = {
    entries: [
        path.resolve(__dirname, "../../crates/rspack/fixtures/basic/entry-a.js"),
        path.resolve(__dirname, "../../crates/rspack/fixtures/basic/entry-b.js"),
    ],
    minify: false,
    entryFileNames: ""
};

(async () => {
    const start = performance.now()
    const rspack = new Rspack(options)
    await rspack.build()
    const end = performance.now()
    console.log(`Build successfully in ${(end - start).toFixed(2)}ms`)
})()