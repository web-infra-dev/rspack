import { promises as fs } from "fs";
import * as path from "path"

import { expect } from "@rspack/test-toolkit"

import { Rspack, RspackPlugin } from "../src/node/rspack"

const fixtureRoot = path.resolve(__dirname, "./fixtures");

describe("rspack", () => {
  it("should work with plugins", async () => {
    const fixture = path.join(fixtureRoot, "rspack-binding");

    const plugin: RspackPlugin = {
      async onLoad(context) {
        if (context.id.endsWith("foo.js")) {
          return {
            content: `${await fs.readFile(context.id, "utf-8")}console.log("fooo");`,
            loader: "js"
          }
        }
      },
      async onResolve(context) {
        if(context.importee === "foo") {
          return {
            uri: path.join(fixture, "foo.js"),
            external: false,
          }
        }
      },
    }

    const rspack = new Rspack({
      entries: [path.join(fixture, "index.js")],
      minify: false,
      entryFileNames: '[name].js',
      outdir: path.join(fixture, "dist"),
      plugins: [plugin],
      sourceMap: false
    });

    await rspack.build();
    expect(await fs.readFile(path.join(fixture, "dist", "index.js"), "utf-8")).toMatchSnapshot();
  })
})