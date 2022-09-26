import assert from "assert";
import { Rspack } from "@rspack/core";
import path from "path";
describe("config", () => {
  it("default config snapshot", () => {
    const resolvedOptions = new Rspack({}).options;
    assert.deepStrictEqual(resolvedOptions.context, process.cwd());
    assert.deepStrictEqual(
      resolvedOptions.dev.static.directory,
      path.resolve(process.cwd(), "./dist")
    );

    // TypeScript will throw `The operand of a 'delete' operator must be optional`.
    // But we remove these configurations with absolute paths.
    delete (resolvedOptions as any).context;
    delete (resolvedOptions as any).dev.static.directory;
    assert.deepStrictEqual(resolvedOptions, {
      mode: "development",
      dev: {
        port: 8080,
        static: {
          watch: false
        },
        devMiddleware: {},
        open: true,
        hmr: false,
        liveReload: true
      },
      entry: {},
      output: {
        assetModuleFilename: undefined,
        chunkFilename: undefined,
        filename: undefined,
        path: undefined,
        publicPath: undefined,
        uniqueName: undefined
      },
      define: {},
      target: ["web"],
      external: {},
      externalType: "",
      plugins: [],
      builtins: { browserslist: [] },
      module: { rules: [] },
      resolve: {
        alias: {},
        preferRelative: false,
        extensions: [".tsx", ".jsx", ".ts", ".js", ".json", ".d.ts"],
        mainFiles: ["index"],
        mainFields: ["module", "main"],
        browserField: true,
        conditionNames: ["module", "import"]
      },
      devtool: false
    });
  });
});
