import path from "node:path";
import { createCompilerCase, describeByWalk } from "@rspack/test-tools";
const srcDir = path.resolve(import.meta.dirname, "./fixtures");

describeByWalk(
	import.meta.filename,
	(name, testConfig, dist) => {
		createCompilerCase(name, srcDir, dist, testConfig);
	},
	{
		level: 1,
		type: "file"
	}
);
