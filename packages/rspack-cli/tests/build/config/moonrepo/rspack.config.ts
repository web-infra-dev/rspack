import packageA_deps from "./packageA/index";
import packageB_deps from "./packageB/index";

import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default {
	mode: "production",
	entry: path.resolve(__dirname, "main.ts"),
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: `moonrepo.bundle.depsA.${packageA_deps.version}-depsB.${packageB_deps.version}.js`
	}
};
