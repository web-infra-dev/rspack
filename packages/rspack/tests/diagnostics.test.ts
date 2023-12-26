import path from "path";
import fs from "fs";
import util from "util";
import { rspack, RspackOptions } from "../src";
import serializer from "jest-serializer-path";
import assert from "assert";
import { ensureRspackConfigNotExist } from "./utils";

expect.addSnapshotSerializer(serializer);

const caseDir = path.resolve(__dirname, "./diagnostics");
const cases = fs.readdirSync(caseDir);

describe("Diagnostics", function () {
	cases.forEach(caseName => {
		it(`${caseName} should compiled and match snapshot`, async function () {
			const casePath = path.resolve(caseDir, caseName);
			ensureRspackConfigNotExist(casePath);
			const outputPath = path.resolve(casePath, `./dist`);
			const configFile = path.resolve(casePath, "webpack.config.js");

			let config = {};
			if (fs.existsSync(configFile)) {
				config = require(configFile);
			}

			let options: RspackOptions = {
				target: "node",
				context: casePath,
				entry: {
					main: "./"
				},
				mode: "development",
				devServer: {
					hot: false
				},
				infrastructureLogging: {
					debug: false
				},
				output: {
					// @ts-ignore
					...config.output,
					path: outputPath
				}
			};

			if (fs.existsSync(outputPath)) {
				fs.rmdirSync(outputPath, { recursive: true });
			}

			const stats = await util.promisify(rspack)(options);
			assert(typeof stats !== "undefined");
			assert(stats.hasErrors() || stats.hasWarnings());
			let output = stats
				.toString({ timings: false, version: false })
				.replace(/\\/g, "/");
			const errorOutputPath = path.resolve(casePath, `./stats.err`);
			const updateSnapshot =
				process.argv.includes("-u") ||
				process.argv.includes("--updateSnapshot");
			if (!fs.existsSync(errorOutputPath) || updateSnapshot) {
				fs.writeFileSync(errorOutputPath, output);
			} else {
				expect(output).toBe(fs.readFileSync(errorOutputPath, "utf-8"));
			}
		});
	});
});
