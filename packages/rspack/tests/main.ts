import { promises as fs } from "fs";
import * as path from "path"
import log from "why-is-node-running"

import { expect, SourceMapConsumer, convertSourceMap } from "@rspack/test-toolkit"

import { Rspack, RspackPlugin } from "../src/node/rspack"

const fixtureRoot = path.resolve(__dirname, "./fixtures");

    // const fixture = path.join(fixtureRoot, "plugin");

    // const plugin: RspackPlugin = {
    //   async onLoad(context) {
    //     if (context.id.endsWith("foo.js")) {
    //       return {
    //         content: `${await fs.readFile(context.id, "utf-8")}console.log("fooo");`,
    //         loader: "js"
    //       }
    //     }
    //   },
    //   async onResolve(context) {
    //     if(context.importee === "foo") {
    //       return {
    //         uri: path.join(fixture, "foo.js"),
    //         external: false,
    //       }
    //     }
    //   },
    // }

    // const rspack = new Rspack({
    //   entries: [path.join(fixture, "index.js")],
    //   minify: false,
    //   entryFileNames: '[name].js',
    //   outdir: path.join(fixture, "dist"),
    //   plugins: [plugin],
    //   sourceMap: false
    // });

    // rspack.build().then(a => {
    //   console.log(a);
    // })

    const fixture = path.join(fixtureRoot, "source-map");

    const plugin = {
      async onResolve(a){
        console.log(a);
        
        return null
      },
      async onLoad(a){
        console.log(a);
        
        return null
      }
    }

    const rspack = new Rspack({
      entries: [path.join(fixture, "index.js")],
      minify: false,
      entryFileNames: '[name].js',
      outdir: path.join(fixture, "dist"),
      sourceMap: true,
      plugins: [plugin]
    });

    rspack.build().then(console.log);
    // const code = await fs.readFile(path.join(fixture, "dist", "index.js"), "utf-8");
  //   // TODO: use `rspack-sources://`, ref: https://webpack.js.org/configuration/output/#outputdevtoolmodulefilenametemplate
  //   // expect(code).toMatchSnapshot();

  //   const sourceMap = convertSourceMap.fromSource(code);
  //   const consumer = await new SourceMapConsumer(sourceMap.toObject())

  //   const meta1 = consumer.originalPositionFor({
  //     line: 3,
  //     column: 4
  //   })
  //   expect(meta1.line).to.eq(1);
  //   expect(meta1.column).to.eq(0);
  //   expect(meta1.source.includes("index.js")).to.be.true;

  //   const meta2 = consumer.originalPositionFor({
  //     line: 4,
  //     column: 4
  //   })
  //   expect(meta2.line).to.eq(2);
  //   expect(meta2.column).to.eq(4);
  //   expect(meta2.source.includes("index.js")).to.be.true;