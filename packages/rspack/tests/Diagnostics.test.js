const path = require("path");
const fs = require("fs");
const util = require("util");
const rspack = require("..");
const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const merge = require("webpack-merge").default;
const assert = require("assert");
const { ensureRspackConfigNotExist } = require("./utils");

expect.addSnapshotSerializer(serializer);

const caseDir = path.resolve(__dirname, "./diagnostics");
const categories = fs.readdirSync(caseDir);

describe("Diagnostics", function () {
	categories.forEach(categoryName => {
		const categoryDir = path.resolve(caseDir, categoryName);
		const cases = fs.readdirSync(categoryDir);
		cases.forEach(caseName => {
			const casePath = path.resolve(categoryDir, caseName);
			describe(caseName, function () {
				it(`${caseName} should compiled and match snapshot`, async function () {
					ensureRspackConfigNotExist(casePath);
					const outputPath = path.resolve(casePath, `./dist`);
					const configFile = path.resolve(casePath, "webpack.config.js");

					let config = {};
					if (fs.existsSync(configFile)) {
						config = require(configFile);
					}

					let options = merge(
						{
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
								path: outputPath
							}
						},
						config
					);

					if (fs.existsSync(outputPath)) {
						fs.rmdirSync(outputPath, { recursive: true });
					}

					const stats = await util.promisify(rspack)(options);
					assert(typeof stats !== "undefined");
					assert(stats.hasErrors() || stats.hasWarnings());
					let output = normalizePaths(
						stats.toString({
							all: false,
							errors: true,
							warnings: true
						})
					);

					// TODO: change to stats.errorStack
					if (casePath.includes("module-build-failed")) {
						// Replace potential loader stack
						output = output
							.replaceAll("│", "")
							.split(/\r?\n/)
							.map(s => s.trim())
							.join("");
					}

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
	});
});
