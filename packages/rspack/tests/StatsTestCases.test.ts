import path from "path";
import fs from "fs";
import util from "util";
import { rspack, RspackOptions } from "../src";
import serializer from "jest-serializer-path";

expect.addSnapshotSerializer(serializer);

const base = path.resolve(__dirname, "statsCases");
const outputBase = path.resolve(__dirname, "stats");
const tests = fs.readdirSync(base).filter(testName => {
	return (
		!testName.startsWith(".") &&
		(fs.existsSync(path.resolve(base, testName, "index.js")) ||
			fs.existsSync(path.resolve(base, testName, "webpack.config.js")))
	);
});
// We do not support externalsType.node-commonjs yet, so I have to use eval to hack around the limitation
function toEval(modName: string) {
	return `eval('require("${modName}")')`;
}
const externalModule = ["uvu", "path", "fs", "expect"];
describe("StatsTestCases", () => {
	tests.forEach(testName => {
		it("should print correct stats for " + testName, async () => {
			const external = Object.fromEntries(
				externalModule.map(x => [x, toEval(x)])
			);
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
					path: outputPath,
					filename: "bundle.js" // not working by now @Todo need fixed later
				},
				externals: external,
				...config // we may need to use deepMerge to handle config merge, but we may fix it until we need it
			};
			const stats = await util.promisify(rspack)(options);
			if (!stats) return expect(false);
			const statsOptions = options.stats ?? { all: true };
			const statsJson = stats.toJson(statsOptions);
			// case ends with error should generate errors
			if (/error$/.test(testName)) {
				expect(statsJson.errors!.length > 0);
			} else {
				expect(statsJson.errors!.length === 0);
			}
			expect(statsJson).toMatchSnapshot();
			const statsString = stats.toString(statsOptions);
			expect(statsString).toMatchSnapshot();
		});
	});
});
