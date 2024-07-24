// @ts-ignore: Because dynamically create {"type": "module"}
import packageA_deps from "./packageA/index.ts";
// @ts-ignore: Because dynamically create {"type": "module"}
import packageB_deps from "./packageB/index.ts";

import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default {
	mode: "production",
	entry: path.resolve(__dirname, "main.ts"),
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: `monorepo.bundle.depsA.${packageA_deps.version}-depsB.${packageB_deps.version}.js`
	}
};
