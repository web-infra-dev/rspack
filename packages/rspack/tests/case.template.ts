import path from "path";
import fs from "fs";
import vm from "vm";
import { Rspack, Plugin, RspackOptions } from "../src";
import assert from "assert";
import createLazyTestEnv from "./helpers/createLazyTestEnv";

// We do not support externalsType.node-commonjs yet, so I have to use eval to hack around the limitation
function toEval(modName: string) {
	return `eval('require("${modName}")')`;
}
// most of these could be removed when we support external builtins by default
const externalModule = ["uvu", "path", "fs", "expect", "source-map"];
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
				if (!fs.existsSync(path.resolve(testRoot, "index.js"))) {
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
							const external = Object.fromEntries(
								externalModule.map(x => [x, toEval(x)])
							);
							const options: RspackOptions = {
								target: ["webworker"], // FIXME when target=commonjs supported
								context: testRoot,
								entry: {
									main: "./"
								},
								output: {
									path: outputPath,
									filename: "bundle.js" // not working by now @Todo need fixed later
								},
								externals: external,
								...config // we may need to use deepMerge to handle config merge, but we may fix it until we need it
							};
							const rspack = new Rspack(options);
							const stats = await rspack.build();
							if (stats.errors.length > 0) {
								console.log(
									`case: ${example}\n\terrors:`,
									`\n\t${stats.errors.map(x => x.message).join("\n")}`
								);
							}
							assert(stats.errors.length === 0);
						});
						// this will run the compiled test code to test against itself, a genius idea from webpack
						it(`${example} should load the compiled test`, async () => {
							const context = {};
							vm.createContext(context);
							const code = fs.readFileSync(bundlePath, "utf-8");
							const fn = vm.runInThisContext(
								`
				(function testWrapper(require,_module,exports,__dirname,__filename,it,expect){
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
								expect
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
