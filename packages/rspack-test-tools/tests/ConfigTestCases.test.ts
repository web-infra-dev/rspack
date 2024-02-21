import path from "path";
import { createConfigCase } from "../src/case/config";
import fs from "fs";

const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/configCases"
);
const tempDir: string = path.resolve(__dirname, "js");

const categories = fs
	.readdirSync(caseDir)
	.filter(folder => !folder.startsWith("_") && !folder.startsWith("."))
	.filter(
		folder =>
			folder.charCodeAt(0) <= "b".charCodeAt(0) &&
			folder.charCodeAt(0) >= "a".charCodeAt(0)
	)
	.map(cat => {
		return {
			name: cat,
			tests: fs
				.readdirSync(path.join(caseDir, cat))
				// .filter(folder => folder === "html-entry-order")
				.filter(folder => !folder.startsWith("_") && !folder.startsWith("."))
				.sort()
		};
	});

describe(`ConfigTestCases`, () => {
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
