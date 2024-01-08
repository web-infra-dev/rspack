import path from "path";
import fs from "fs";
import vm from "vm";
import { rspack, RspackOptions } from "../src";
import assert from "assert";
import createLazyTestEnv from "./helpers/createLazyTestEnv";
import { isValidTestCaseDir } from "./utils";
const checkArrayExpectation = require("./checkArrayExpectation");

// most of these could be removed when we support external builtins by default
export function describeCases(config: { name: string; casePath: string }) {
	const casesPath = path.resolve(__dirname, config.casePath);
	let categoriesDir = fs.readdirSync(casesPath);
	let categories = categoriesDir
		.filter(x => isValidTestCaseDir(x) && x !== "dist")
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.resolve(casesPath, cat))
					.filter(isValidTestCaseDir)
			};
		});
	describe(config.name, () => {
		categories.forEach(category => {
			category.tests
				.filter(test => {
					const testDirectory = path.join(casesPath, category.name, test);
					const filterPath = path.join(testDirectory, "test.filter.js");
					if (fs.existsSync(filterPath) && !require(filterPath)(config)) {
						describe.skip(test, () => {
							it("filtered", () => {});
						});
						return false;
					}
					return true;
				})
				.forEach(example => {
					const testRoot = path.resolve(
						casesPath,
						`./${category.name}/${example}/`
					);
					const outputPath = path.resolve(testRoot, `./dist`);
					const bundlePath = path.resolve(outputPath, "main.js");

					if (
						![".js", ".jsx", ".ts", ".tsx", ".mjs"].every(ext => {
							return !fs.existsSync(path.resolve(testRoot, "index" + ext));
						})
					) {
						describe(category.name, () => {
							describe(example, () => {
								it(`${example} should compile`, done => {
									const configFile = path.resolve(
										testRoot,
										"webpack.config.js"
									);
									let config = {};
									if (fs.existsSync(configFile)) {
										config = require(configFile);
									}
									const options: RspackOptions = {
										target: "node",
										context: testRoot,
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
										...config, // we may need to use deepMerge to handle config merge, but we may fix it until we need it
										output: {
											// @ts-ignore
											...config.output,
											path: outputPath
										}
									};
									if (fs.existsSync(outputPath)) {
										fs.rmdirSync(outputPath, { recursive: true });
									}
									rspack(options, (err, stats) => {
										if (err) {
											return done(err);
										}
										const statsJson = stats!.toJson();
										if (
											checkArrayExpectation(
												testRoot,
												statsJson,
												"error",
												"Error",
												done
											)
										) {
											return;
										}
										if (
											checkArrayExpectation(
												testRoot,
												statsJson,
												"warning",
												"Warning",
												done
											)
										) {
											return;
										}

										Promise.resolve().then(done);
									});
								});
								// this will run the compiled test code to test against itself, a genius idea from webpack
								it(`${example} should load the compiled test`, async () => {
									const context = {};
									vm.createContext(context);
									const code = fs.readFileSync(bundlePath, "utf-8");
									const fn = vm.runInThisContext(
										`
				(function testWrapper(require,_module,exports,__dirname,__filename,it,expect,jest, define){
          global.expect = expect;
					function nsObj(m) { Object.defineProperty(m, Symbol.toStringTag, { value: "Module" }); return m; }
				  ${code};
				 }
				)
				`,
										bundlePath
									);
									const m = {
										exports: {}
									};
									fn.call(
										m.exports,
										function (p) {
											return p && p.startsWith(".")
												? require(path.resolve(outputPath, p))
												: require(p);
										},
										m,
										m.exports,
										outputPath,
										bundlePath,
										_it,
										expect,
										jest
									);
									return m.exports;
								});
							});
						});
						const { it: _it } = createLazyTestEnv(10000);
					}
				});
		});
	});
}
