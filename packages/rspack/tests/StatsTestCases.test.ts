import path from "path";
import fs from "fs";
import util from "util";
import { rspack, RspackOptions } from "../src";
import serializer from "jest-serializer-path";
import { isValidTestCaseDir } from "./utils";
import { replace } from "./lib/util/replaceMitteDiagnostic";

expect.addSnapshotSerializer(serializer);

const base = path.resolve(__dirname, "statsCases");
const tests = fs.readdirSync(base).filter(testName => {
	return (
		isValidTestCaseDir(testName) &&
		(fs.existsSync(path.resolve(base, testName, "index.js")) ||
			fs.existsSync(path.resolve(base, testName, "webpack.config.js")))
	);
});

describe("StatsTestCases", () => {
	tests.forEach(testName => {
		it("should print correct stats for " + testName, async () => {
			const context = path.resolve(base, testName);
			const outputPath = path.resolve(base, testName, "dist");
			const configPath = path.resolve(base, testName, "webpack.config.js");
			let config = {};
			if (fs.existsSync(configPath)) {
				config = require(configPath);
			}
			let options;
			if (Array.isArray(config)) {
				options = config.map(c => {
					const result: RspackOptions = {
						target: "node",
						context,
						entry: {
							main: "./index"
						},
						output: {
							filename: "bundle.js"
						},
						...c
					};
					result.output!.path = outputPath;
					return result;
				});
			} else {
				options = {
					target: "node",
					context,
					entry: {
						main: "./index"
					},
					output: {
						filename: "bundle.js"
					},
					...config
				};
				options.output!.path = outputPath;
			}
			const stats = await util.promisify(rspack)(options);
			if (!stats) return expect(false);
			const statsOptions = options.stats ?? {
				all: true,
				timings: false,
				builtAt: false,
				version: false
			};
			if (typeof statsOptions === "object" && statsOptions !== null) {
				Object.assign(statsOptions, {
					timings: false,
					builtAt: false,
					version: false
				});
			}
			const statsJson = stats.toJson(statsOptions);
			// case ends with error should generate errors
			if (/error$/.test(testName)) {
				expect(statsJson.errors!.length > 0);
			} else if (statsJson.errors) {
				expect(statsJson.errors.length === 0);
			}

			statsJson.errors?.forEach(e => {
				e.message = replace(e.message);
				e.formatted = replace(e.formatted);
			});
			statsJson.warnings?.forEach(e => {
				e.message = replace(e.message);
				e.formatted = replace(e.formatted);
			});

			expect(statsJson).toMatchSnapshot();
			let statsString = stats.toString(statsOptions);
			expect(replace(statsString)).toMatchSnapshot();
		});
	});
});
