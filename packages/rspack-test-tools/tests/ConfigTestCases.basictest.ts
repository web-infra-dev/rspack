import path from "path";
import { createConfigCase } from "../src/case/config";
import fs from "fs";
import { isDirectory, isValidCaseDirectory } from "../src/helper";

const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/configCases"
);

const tempDir: string = path.resolve(__dirname, "../../rspack/tests/js");

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

describe(`ConfigTestCases`, () => {
	for (let { name: catName, tests } of categories) {
		describe(catName, () => {
			for (const testName of tests) {
				const src = path.join(caseDir, catName, testName);
				const dist = path.join(tempDir, "ConfigTestCases", catName, testName);
				createConfigCase(testName, src, dist);
			}
		});
	}
});
