import path from "path";
const yargs: typeof import("yargs") = require("yargs");
import { run } from "./build";

// build({ main: path.resolve(__dirname, '../fixtures/index.js') });

yargs
  .scriptName("rspack")
  .usage("$0 <root>")
  .command(
    "$0 [root]",
    "start dev server",
    (yargs) => {
      yargs.positional("root", {
        type: "string",
        default: process.cwd(),
        describe: "project root",
      });
    },
    (argv: any) => {
      const root = path.resolve(process.cwd(), argv.root);
      console.log("root:", root);
      const pakgPath = path.resolve(root, "package.json");
      const pkg = require(pakgPath);
      let entry = pkg?.rspack?.entry;
      let manualChunk = pkg?.rspack?.manualChunks;
      if (!entry) {
        entry = {
          main: path.resolve(root, "index.js"),
        };
      }
      for (const [key, value] of Object.entries(entry)) {
        entry[key] = path.resolve(root, value as string);
      }
      let alias = pkg?.rspack?.alias ?? {};
      alias = Object.fromEntries(
        Object.entries(alias).map(([key, value]) => [
          key,
          (value as string).replace("<ROOT>", process.cwd),
        ])
      );
      console.log("pkg:", pkg?.rspack);
      run({
        entry: entry,
        root: root,
        manualChunks: manualChunk ?? {},
        loader: pkg?.rspack?.loader,
        inlineStyle: pkg?.rspack?.inlineStyle,
        alias: alias,
        react: {
          refresh: pkg?.rspack?.react?.refresh,
        },
        sourceMap: pkg?.rspack?.sourcemap ?? true,
        codeSplitting: pkg?.rspack?.splitting ?? true,
        svgr: pkg?.rspack?.svgr ?? true,
      });
    }
  )
  .help().argv;
