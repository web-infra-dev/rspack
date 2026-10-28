import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description: "stages complete chunks until apply and resolves concurrent imports",
	options(context) {
		return {
			context: context.getDist("input"),
			entry: "./index.js",
			mode: "development",
			devtool: false,
			target: "web",
			output: {
				path: context.getDist("output"),
				filename: "main.js",
				chunkFilename: "[name].js",
				publicPath: "/"
			},
			optimization: {
				moduleIds: "named",
				chunkIds: "named",
				usedExports: false,
				concatenateModules: false,
				removeAvailableModules: false,
				splitChunks: {
					cacheGroups: {
						default: false,
						defaultVendors: false,
						shared: {
							test: /[\\/](dep|z)\.js$/,
							name: "shared",
							chunks: "async",
							enforce: true
						}
					}
				}
			},
			plugins: [new rspack.HotModuleReplacementPlugin()]
		};
	},
	async build(context, compiler) {
		const input = context.getDist("input");
		fs.mkdirSync(input, { recursive: true });
		const write = (file, source) => fs.writeFileSync(path.join(input, file), source);
		write("index.js", `
			globalThis.req = __webpack_require__;
			globalThis.hot = module.hot;
			globalThis.readDep = () => require('./dep');
			globalThis.loadPage = () => import(/* webpackChunkName: 'page' */ './page');
			globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');
			module.hot.accept();
		`);
		write("page.js", "module.exports = 0; module.hot.accept();");
		write("dep.js", "module.exports = 2; module.hot.accept();");
		write("z.js", "module.exports = 99;");
		write("other.js", "module.exports = [require('./dep'), require('./z')];");
		const builds = [];
		for (let step = 0; step < 2; step++) {
			if (step) {
				write("page.js", "module.exports = require('./z'); module.hot.accept();");
				write("dep.js", "module.exports = 3; module.hot.accept();");
			}
			const stats = await new Promise((resolve, reject) => compiler.run(
				(error, stats) => error ? reject(error) : resolve(stats),
				{ modifiedFiles: new Set(step ? ["page.js", "dep.js"].map(file => path.join(input, file)) : []) }
			));
			assert.equal(stats.hasErrors(), false, stats.toString());
			builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
		}
		for (const abort of [false, true]) {
			let assets = builds[0];
			const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
			vm.runInContext(assets["main.js"], sandbox);
			const { req, hot } = sandbox;
			let hold = false;
			const queued = [];
			const requests = [];
			req.l = (url, done) => {
				const name = path.basename(url);
				requests.push(name);
				const execute = () => {
					vm.runInContext(assets[name], sandbox);
					done({ type: "load" });
				};
				if (hold && !name.includes(".hot-update.")) queued.push(execute);
				else execute();
			};
			await sandbox.loadPage();
			// Prefetching records null without installing factories. It still needs
			// a complete payload when the loaded page gains this dependency.
			if (abort) req.hmrS_jsonp.shared = null;
			assets = builds[1];
			req.hmrM = async () => JSON.parse(assets[`main.${req.h()}.hot-update.json`]);
			await hot.check(false);
			assert.equal(hot.status(), "ready");
			assert.equal(sandbox.readDep(), 2);
			assert(requests.some(name => name.endsWith(".full.js")));
			if (abort) {
				req.c["./page.js"].hot.decline();
				req.c["./page.js"].hot.invalidate();
				await assert.rejects(hot.apply(), /self decline/);
				assert.equal(sandbox.readDep(), 2);
				assert.equal(req.hmrS_jsonp.shared, null);
				continue;
			}
			let pending;
			hot.addStatusHandler(status => {
				if (status !== "dispose") return;
				hold = true;
				pending = sandbox.loadOther();
			});
			await hot.apply();
			assert.equal(sandbox.readDep(), 3);
			assert.equal(queued.length, 2);
			queued.forEach(execute => execute());
			let timer;
			try {
				const other = await Promise.race([
					pending,
					new Promise((_, reject) => { timer = setTimeout(() => reject(new Error("import did not resolve")), 1000); })
				]);
				assert.equal(other.default[0], 3);
				assert.equal(other.default[1], 99);
			} finally {
				clearTimeout(timer);
			}
			assert.equal(hot.status(), "idle");
		}
	}
};
