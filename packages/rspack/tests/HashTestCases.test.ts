import path from "path";
import fs from "fs";
import util from "util";
import { rspack } from "../src";
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

const base = path.resolve(__dirname, "hashCases");
const tests = fs.readdirSync(base).filter(testName => {
	return (
		!testName.startsWith(".") &&
		(fs.existsSync(path.resolve(base, testName, "index.js")) ||
			fs.existsSync(path.resolve(base, testName, "webpack.config.js")))
	);
});

describe("HashTestCases", () => {
	tests.forEach(testName => {
		it("should print correct hash for " + testName, async () => {
			const configPath = path.resolve(base, testName, "webpack.config.js");
			const testConfigPath = path.resolve(base, testName, "test.config.js");
			let config;
			if (fs.existsSync(configPath)) {
				config = require(configPath);
			} else {
				throw new Error("HashTestCases must have a webpack.config.js");
			}
			let testConfig;
			if (fs.existsSync(testConfigPath)) {
				testConfig = require(testConfigPath);
			}
			const stats = await util.promisify(rspack)(config);
			if (!stats) return expect(false);
			const statsJson = stats.toJson({ assets: true });
			// case ends with error should generate errors
			if (/error$/.test(testName)) {
				expect(statsJson.errors!.length > 0);
			} else {
				expect(statsJson.errors!.length === 0);
			}
			const files = statsJson.assets?.map(x => x.name);
			expect(files).toMatchSnapshot();

			if (testConfig && testConfig.validate) {
				testConfig.validate(stats);
			}
		});
	});
});
