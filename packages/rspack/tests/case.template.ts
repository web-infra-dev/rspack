import path from "path";
import fs from "fs";
import vm from "vm";
import util from "util";
import { rspack, RspackOptions } from "../src";
import assert from "assert";
import createLazyTestEnv from "./helpers/createLazyTestEnv";

const define = function (...args) {
	const factory = args.pop();
	factory();
};

// most of these could be removed when we support external builtins by default
export function describeCases(config: { name: string; casePath: string }) {
	const casesPath = path.resolve(__dirname, config.casePath);
	let categoriesDir = fs.readdirSync(casesPath);
	let categories = categoriesDir
		.filter(x => x !== "dist" && !x.startsWith("."))
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.resolve(casesPath, cat))
					.filter(folder => !folder.includes("_") && !folder.startsWith("."))
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
					[".js", ".jsx", ".ts", ".tsx"].every(ext => {
						return !fs.existsSync(path.resolve(testRoot, "index" + ext));
					})
				) {
					continue;
				}
				describe(category.name, () => {
					describe(example, () => {
						it(`${example} should compile`, async () => {
							const configFile = path.resolve(testRoot, "webpack.config.js");
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
									publicPath: "/",
									// @ts-ignore
									...config.output,
									path: outputPath
								}
							};
							const stats = await util.promisify(rspack)(options);
							const statsJson = stats!.toJson();
							if (category.name === "errors") {
								assert(statsJson.errors!.length > 0);
							} else if (category.name === "warnings") {
								assert(statsJson.warnings!.length > 0);
							} else {
								if (statsJson.errors!.length > 0) {
									console.log(
										`case: ${example}\nerrors:\n`,
										`${statsJson.errors!.map(x => x.message).join("\n")}`
									);
								}
								assert(statsJson.errors!.length === 0);
							}
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
								jest,
								define
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
