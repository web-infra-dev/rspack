import path from "path";
import { createConfigCase } from "../src/case/config";
import fs from "fs";
import { isDirectory, isValidCaseDirectory } from "../src/helper";

const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/configCases"
);
// const tempDir: string = path.resolve(__dirname, "js");

const categories = fs
	.readdirSync(caseDir)
	.filter(isValidCaseDirectory)
	.filter(folder => isDirectory(path.join(caseDir, folder)))
	.map(cat => {
		return {
			name: cat,
			tests: fs
				.readdirSync(path.join(caseDir, cat))
				.filter(isValidCaseDirectory)
				.filter(folder => isDirectory(path.join(caseDir, cat, folder)))
				.sort()
		};
	});

describe(`configCases`, () => {
	for (let { name, tests } of categories) {
		describe(name, () => {
			for (const testName of tests) {
				const src = path.join(caseDir, name, testName);
				const dist = path.join(src, "dist");
				createConfigCase(testName, src, dist);
			}
		});
	}
});
