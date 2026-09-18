import path from "node:path";
import { createErrorCase, describeByWalk } from "@rspack/test-tools";
const caseDir = path.resolve(import.meta.dirname, "./errorCases");

describeByWalk(
	import.meta.filename,
	(name, testConfig, dist) => {
		createErrorCase(name, caseDir, dist, testConfig);
	},
	{
		level: 1,
		type: "file"
	}
);
