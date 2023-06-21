import path from "path";
import fs from "fs";
import util from "util";
import { rspack, RspackOptions, cleverMerge } from "../src";
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

const project_dir_reg = new RegExp(
	path.join(__dirname, "..").replace(/\\/g, "\\\\"),
	"g"
);

const base = path.resolve(__dirname, "statsCases");
const outputBase = path.resolve(__dirname, "stats");
const tests = fs.readdirSync(base).filter(testName => {
	return (
		!testName.startsWith(".") &&
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
			const options: RspackOptions = {
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
			const stats = await util.promisify(rspack)(options);
			if (!stats) return expect(false);
			const statsOptions = options.stats ?? {
				all: true,
				timings: false,
				builtAt: false
			};
			const statsJson = stats.toJson(statsOptions);
			// case ends with error should generate errors
			if (/error$/.test(testName)) {
				expect(statsJson.errors!.length > 0);
			} else if (statsJson.errors) {
				expect(statsJson.errors.length === 0);
			}
			statsJson.errors?.forEach(error => {
				error.formatted = error.formatted
					?.replace(project_dir_reg, "<PROJECT_ROOT>")
					?.replace(/\\/g, "/");
			});
			statsJson.warnings?.forEach(error => {
				error.formatted = error.formatted
					?.replace(project_dir_reg, "<PROJECT_ROOT>")
					?.replace(/\\/g, "/");
			});
			expect(statsJson).toMatchSnapshot();
			let statsString = stats.toString(statsOptions);
			statsString = statsString
				.replace(project_dir_reg, "<PROJECT_ROOT>")
				.replace(/\\/g, "/");
			expect(statsString).toMatchSnapshot();
		});
	});
});
