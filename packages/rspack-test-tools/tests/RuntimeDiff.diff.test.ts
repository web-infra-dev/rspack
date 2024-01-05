import { globSync } from "glob";
import path from "path";
import { createDiffCase } from "../src/case/diff";

const caseDir: string = path.resolve(__dirname, "runtimeDiffCases");
const tempDir: string = path.resolve(__dirname, "js");

describe(`RuntimeDiffCases`, () => {
	for (let name of globSync("**/test.config.js", {
		cwd: caseDir
	})) {
		name = path.dirname(name);
		const src = path.join(caseDir, name);
		const dist = path.join(tempDir, "runtime-diff", name);
		createDiffCase(name, src, dist);
	}
});
