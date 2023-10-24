import path from "path";
import fs from "fs-extra";
import { createDiffCase } from "./helpers/tester";

const caseDir: string = path.resolve(__dirname, "runtimeDiffCases");
const tempDir: string = path.resolve(__dirname, "js");
const cases: string[] = fs
	.readdirSync(caseDir)
	.filter(testName => !testName.startsWith("."));

describe("RuntimeDiffCases", () => {
	cases.forEach(name => {
		const src = path.join(caseDir, name);
		const dist = path.join(tempDir, `runtime-diff/${name}`);
		createDiffCase(name, src, dist);
	});
});
