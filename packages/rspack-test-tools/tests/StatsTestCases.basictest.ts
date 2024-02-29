import path from "path";
import fs from "fs";
import { createStatsCase } from "../src/case/stats";

const NAME = "StatsTestCases";
const caseDir: string = path.resolve(
	__dirname,
	"../../rspack/tests/statsCases"
);
const distDir: string = path.resolve(__dirname, `../../rspack/tests/js/stats`);

const tests = fs
	.readdirSync(caseDir)
	.filter(
		testName =>
			fs.existsSync(path.join(caseDir, testName, "index.js")) ||
			fs.existsSync(path.join(caseDir, testName, "webpack.config.js"))
	);

describe(NAME, () => {
	jest.setTimeout(30000);
	for (const name of tests) {
		const src = path.join(caseDir, name);
		const dist = path.join(distDir, name);
		createStatsCase(name, src, dist);
	}
});
