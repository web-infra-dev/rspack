import fs from "node:fs";
import path from "node:path";
import { runCompiler } from "@rspack/test-tools/helper/lifecycle";

function rebuild(compiler, changed = []) {
  return new Promise((resolve, reject) => {
    compiler.__internal__rebuild(new Set(changed), new Set(), (error) => {
      if (error) reject(error);
      else resolve(compiler._lastCompilation);
    });
  });
}

function cacheCase(newCache, incremental, concatenateModules) {
  let root;
  function write(effectful) {
    for (const name of ["direct", "transitive"]) {
      fs.writeFileSync(
        path.join(root, `${name}.js`),
        `${effectful ? `globalThis.${name}Effect = true;` : ""}\nexport let value = 42;\n`,
      );
    }
  }
  function check(compilation, pure, declaredPure = true) {
    expect(compilation.errors.map((error) => error.message)).toEqual([]);
    const source = compilation
      .getAssets()
      .map((asset) => asset.source.source().toString())
      .join("\n");
    if (concatenateModules) expect(source).toContain("// EXTERNAL MODULE:");
    for (const name of ["direct", "transitive", "declared/index"]) {
      expect(
        source.includes(`/*#__PURE__*/ __webpack_require__("./${name}.js")`),
      ).toBe(name === "declared/index" ? declaredPure : pure);
    }
  }
  return {
    description: `should refresh import annotations with memory cache (newCache=${newCache}, incremental=${incremental}, concatenate=${concatenateModules})`,
    options(context) {
      root = context.getDist(
        `src-${newCache}-${incremental}-${concatenateModules}`,
      );
      fs.mkdirSync(root, { recursive: true });
      fs.writeFileSync(
        path.join(root, "index.js"),
        'import { value as direct } from "./direct"; import { value as transitive } from "./barrel"; import { value as declared } from "./declared"; import { consume } from "./local"; consume(direct, transitive, declared);',
      );
      fs.writeFileSync(
        path.join(root, "local.js"),
        "export function consume(...values) { console.log(...values); }",
      );
      fs.writeFileSync(
        path.join(root, "barrel.js"),
        'export { value } from "./transitive";',
      );
      fs.mkdirSync(path.join(root, "declared"), { recursive: true });
      fs.writeFileSync(
        path.join(root, "declared/index.js"),
        "globalThis.declaredEffect = true; export let value = 42;",
      );
      fs.writeFileSync(
        path.join(root, "declared/package.json"),
        '{"sideEffects":false}',
      );
      write(false);
      return {
        context: root,
        entry: "./index.js",
        mode: "development",
        devtool: false,
        cache: true,
        incremental,
        experiments: { newCache },
        output: {
          path: context.getDist("output"),
          module: true,
          library: { type: "module" },
          filename: "[name].mjs",
          chunkFilename: "[name].mjs",
          pathinfo: false,
        },
        optimization: {
          moduleIds: "named",
          providedExports: true,
          sideEffects: true,
          usedExports: false,
          inlineExports: false,
          concatenateModules,
          // Keep the targets outside the concatenation group to exercise its require initializers.
          splitChunks: concatenateModules
            ? {
                chunks: "all",
                minSize: 0,
                cacheGroups: {
                  shared: {
                    test: /(?:direct|transitive)\.js$|[/\\]declared[/\\]index\.js$/,
                    name: "shared",
                    enforce: true,
                  },
                },
              }
            : false,
        },
      };
    },
    async build(_context, compiler) {
      check((await runCompiler(compiler)).compilation, true);
      check(await rebuild(compiler), true);
      const changed = ["direct", "transitive"].map((name) =>
        path.join(root, `${name}.js`),
      );
      write(true);
      check(await rebuild(compiler, changed), false);
      write(false);
      check(await rebuild(compiler, changed), true);
      if (!incremental) {
        // Refactorize package metadata without recovering the previous module graph.
        const metadata = path.join(root, "declared/package.json");
        fs.writeFileSync(metadata, '{"sideEffects":true}');
        check(await rebuild(compiler, [metadata]), true, false);
        fs.writeFileSync(metadata, '{"sideEffects":false}');
        check(await rebuild(compiler, [metadata]), true, true);
      }
    },
  };
}

export default [false, true].flatMap((newCache) =>
  [false, true].flatMap((incremental) =>
    [false, true].map((concatenate) =>
      cacheCase(newCache, incremental, concatenate),
    ),
  ),
);
