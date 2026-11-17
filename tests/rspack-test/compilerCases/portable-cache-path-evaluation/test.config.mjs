import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import { closeCompiler, runCompiler } from "@rspack/test-tools/helper/lifecycle";
import { removeTree } from "../_remove-tree.mjs";

const loader = `const path = require("node:path");
module.exports = function () {
  this.experiments.portableOutput({ paths: [this.resourcePath] });
  return 'const U = ' + JSON.stringify(this.resourcePath) +
    '.indexOf(' + JSON.stringify(path.join("/a/proj/")) + ') !== -1 ? URL : null;' +
    'export function load() { return new U("./icon.png", import.meta.url); }';
};`;

async function build(context, cache = { type: "persistent", portable: true }) {
  const compiler = rspack({
    context,
    mode: "development",
    devtool: false,
    entry: "./src/index.js",
    output: { path: path.join(context, "dist"), clean: true, publicPath: "" },
    module: { rules: [{ test: /value\.js$/, loader: "./loader.cjs" }] },
    cache,
    infrastructureLogging: { level: "error" }
  });
  try {
    const stats = await runCompiler(compiler);
    if (stats.hasErrors()) throw new Error(stats.toString({ all: false, errors: true }));
    return {
      built: [...stats.compilation.builtModules].some((module) =>
        module.identifier().endsWith(path.join("src", "value.js"))),
      assets: Object.fromEntries(stats.compilation.getAssets().map(({ name }) =>
        [name, fs.readFileSync(path.join(context, "dist", name)).toString("base64")]))
    };
  } finally {
    await closeCompiler(compiler);
  }
}

export default {
  description: "binds root-dependent alias selection even when the renamed initializer is not walked",
  async run() {
    const temp = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), "rspack-path-eval-")));
    try {
      const writer = path.join(temp, "a", "proj");
      const reader = path.join(temp, "longer-reader", "proj");
      fs.mkdirSync(path.join(writer, ".git"), { recursive: true });
      fs.mkdirSync(path.join(writer, "src"));
      fs.writeFileSync(path.join(writer, "package.json"), '{"name":"path-evaluation"}');
      fs.writeFileSync(path.join(writer, "loader.cjs"), loader);
      fs.writeFileSync(path.join(writer, "src", "index.js"), 'export { load } from "./value.js";');
      fs.writeFileSync(path.join(writer, "src", "value.js"), "");
      fs.writeFileSync(path.join(writer, "src", "icon.png"), "asset bytes");
      const first = await build(writer);
      expect(Object.keys(first.assets).some((name) => name.endsWith(".png"))).toBe(true);
      fs.mkdirSync(path.dirname(reader), { recursive: true });
      fs.cpSync(writer, reader, { recursive: true });
      const moved = await build(reader);
      expect(moved.built).toBe(true);
      expect(Object.keys(moved.assets).some((name) => name.endsWith(".png"))).toBe(false);
      const uncached = await build(reader, false);
      expect(moved.assets).toEqual(uncached.assets);
      removeTree(path.join(reader, "node_modules"));
      const cold = await build(reader);
      expect(moved.assets).toEqual(cold.assets);
    } finally {
      removeTree(temp);
    }
  }
};
