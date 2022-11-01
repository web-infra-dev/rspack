import path from "path";
import fs from "fs";
import vm from "vm";
import util from "util";
import { rspack, RspackOptions } from "../src";
import assert from "assert";
import createLazyTestEnv from "./helpers/createLazyTestEnv";
import deepmerge from "deepmerge";

// most of these could be removed when we support external builtins by default
const externalModule = ["uvu", "path", "fs", "expect", "source-map", "util"];
export function describeCases(config: { name: string; casePath: string }) {
	const casesPath = path.resolve(__dirname, config.casePath);
	let categoriesDir = fs.readdirSync(casesPath);
	let categories = categoriesDir
		.filter(x => x !== "dist" || x.includes("."))
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.resolve(casesPath, cat))
					.filter(folder => !folder.includes("_"))
			};
		});
	describe(config.name, () => {
		for (const category of categories) {
			for (const example of category.tests) {
				const testRoot = path.resolve(
					casesPath,
					`./${category.name}/${example}/`
				);
				const outputPath = path.resolve(testRoot, `./dist`);
				const bundlePath = path.resolve(outputPath, "main.js");
				if (
					!(
						fs.existsSync(path.resolve(testRoot, "index.js")) ||
						fs.existsSync(path.resolve(testRoot, "index.jsx"))
					)
				) {
					continue;
				}
				describe(category.name, () => {
					describe(example, () => {
						it(`${example} should compile`, async () => {
							const configFile = path.resolve(testRoot, "webpack.config.js");
							let config: RspackOptions = {};
							if (fs.existsSync(configFile)) {
								config = require(configFile);
							}
							const externals = Object.fromEntries(
								externalModule.map(x => [x, x])
							);
							const options: RspackOptions = deepmerge(config, {
								target: "node", // FIXME when target=commonjs supported
								context: testRoot,
								entry: {
									main: "./"
								},
								mode: "development",
								output: {
									path: outputPath
								},
								infrastructureLogging: {
									debug: true
								},
								externals,
								externalsType: "node-commonjs",
								...config // we may need to use deepMerge to handle config merge, but we may fix it until we need it
							});
							const stats = await util.promisify(rspack)(options);
							const statsJson = stats.toJson();
							if (statsJson.errors.length > 0) {
								console.log(
									`case: ${example}\nerrors:\n`,
									`${statsJson.errors.map(x => x.message).join("\n")}`
								);
							}
							assert(statsJson.errors.length === 0);
						});
						// this will run the compiled test code to test against itself, a genius idea from webpack
						it(`${example} should load the compiled test`, async () => {
							const context = {};
							vm.createContext(context);
							const code = fs.readFileSync(bundlePath, "utf-8");
							const fn = vm.runInThisContext(
								`
				(function testWrapper(require,_module,exports,__dirname,__filename,it,expect,jest){
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
		}
	});
}
