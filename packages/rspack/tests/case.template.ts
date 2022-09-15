import { test, suite } from "uvu";
import path from "path";
import fs from "fs";
import vm from "vm";
import { expect } from "expect";

import { Rspack, Plugin, RspackOptions } from "../src";

const casesPath = path.resolve(__dirname, "cases");
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
// We do not support externalsType.node-commonjs yet, so I have to use eval to hack around the limitation
function toEval(modName: string) {
	return `eval('require("${modName}")')`;
}
// most of these could be removed when we support external builtins by default
const externalModule = ["uvu", "path", "fs", "expect"];
export function runCase(config: { name: string }) {
	for (const category of categories) {
		for (const example of category.tests) {
			const entry = `./${category.name}/${example}/`;
			const outputPath = path.resolve(
				casesPath,
				`./${category.name}/${example}/dist`
			);
			const bundlePath = path.resolve(outputPath, "main.js");
			test(`${example} should compile`, async () => {
				const external = Object.fromEntries(
					externalModule.map(x => [x, toEval(x)])
				);
				const options: RspackOptions = {
					target: ["webworker"], // FIXME when target=commonjs supported
					context: casesPath,
					entry: {
						main: entry
					},
					output: {
						path: outputPath,
						filename: "bundle.js" // not working by now @Todo need fixed later
					},
					external: external
				};
				const rspack = new Rspack(options);
				const stats = await rspack.build();
				if (stats.errors.length > 0) {
					throw new Error(stats.errors.map(x => x.message).join("\n"));
				}
			});
			// this will run the compiled test code to test against itself, a genius idea from webpack
			test(`${example} should load the compiled test`, async () => {
				const context = {};
				vm.createContext(context);
				const code = fs.readFileSync(bundlePath, "utf-8");
				function _require() {}
				try {
					const fn = vm.runInThisContext(
						`
				(function testWrapper(require,module,exports,__dirname,__filename,it,expect){
					global.expect = expect;
					function nsObj(m) { Object.defineProperty(m, Symbol.toStringTag, { value: "Module" }); return m; }
				  ${code};
					it.run();
				 }
				)
				`,
						bundlePath
					);
					const m = {
						exports: {}
					};
					let it = suite(example);
					fn.call(
						m.exports,
						_require,
						m,
						m.exports,
						outputPath,
						bundlePath,
						it,
						expect
					);
				} catch (err) {
					console.log("err:", err);
				}
			});
			test.run();
		}
	}
}
