const path = require("path");
const fs = require("fs");
const util = require("util");
const { rspack } = require("../dist");
const serializer = require("jest-serializer-path");

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
			const testPath = path.resolve(base, testName);
			const configPath = path.resolve(testPath, "webpack.config.js");
			const testConfigPath = path.resolve(testPath, "test.config.js");
			let config;
			if (fs.existsSync(configPath)) {
				config = require(configPath);
			} else {
				throw new Error("HashTestCases must have a webpack.config.js");
			}
			const isMultiCompiler = Array.isArray(config);
			if (!isMultiCompiler && typeof config === "object") {
				config.context ??= testPath;
				config.entry ??= "./index";
				config.output ??= {};
				config.output.path ??= path.resolve(testPath, "dist");
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
				expect(statsJson.errors.length > 0);
			} else {
				expect(statsJson.errors.length === 0);
			}

			if (testConfig && testConfig.validate) {
				testConfig.validate(stats);
			} else {
				throw new Error(
					"HashTestCases should have test.config.js and a validate method"
				);
			}
		});
	});
});
