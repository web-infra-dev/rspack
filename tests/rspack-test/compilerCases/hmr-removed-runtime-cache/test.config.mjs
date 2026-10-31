import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	description: "disposes modules that lose their loaded chunk and preserves modules supplied by another loaded chunk",
	options(context) {
		return {
			context: context.getDist("input"),
			entry: { a: "./a.js", b: "./b.js" },
			mode: "development",
			devtool: false,
			target: "web",
			output: {
				path: context.getDist("output"),
				filename: "[name].js",
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
							test: /[\\/]dep\.js$/,
							name: "shared",
							chunks: chunk => ["a", "b", "page", "z"].includes(chunk.name),
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
		const write = (name, source) => fs.writeFileSync(path.join(input, name), source);
		// Cover immediate/later edits and both ordinary and supplemental replacements.
		for (const mode of ["async", "supplement", "initial"]) {
			const initial = mode === "initial";
			const supplement = mode === "supplement";
			const builds = [];
			for (let step = 0; step < 3; step++) {
				write("a.js", `
					globalThis.req = __webpack_require__;
					globalThis.hot = module.hot;
					${initial || supplement ? "require('./page');" : ""}
					${!initial && !step ? `globalThis.loadPage = () => import(/* webpackChunkName: '${supplement ? "old" : "page"}' */ './${supplement ? "old" : "page"}');` : ""}
					${supplement ? "globalThis.loadZ = () => import(/* webpackChunkName: 'z' */ './dep');" : ""}
					globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');
					module.hot.accept();
				`);
				write("b.js", `${supplement ? "globalThis.loadOld = () => import(/* webpackChunkName: 'old' */ './old');" : "require('./dep');"} module.hot.accept();`);
				write("page.js", `globalThis.pageValue = ${supplement ? step ? "require('./dep')" : "0" : step ? "9" : "require('./dep')"}; module.hot.accept();`);
				write("dep.js", `module.exports = ${step && (initial || step === 2) ? 3 : 2}; module.hot.accept();`);
				write("other.js", "module.exports = require('./dep');");
				write("old.js", "module.exports = require('./dep');");
				const stats = await new Promise((resolve, reject) => compiler.run(
					(error, stats) => error ? reject(error) : resolve(stats),
					{ modifiedFiles: new Set(["a.js", "b.js", "page.js", "dep.js", "other.js", "old.js"].map(name => path.join(input, name))) }
				));
				assert.equal(stats.hasErrors(), false, stats.toString());
				builds.push(Object.fromEntries(stats.compilation.getAssets().map(asset => [asset.name, asset.source.source().toString()])));
			}
			for (const otherLoaded of mode === "async" ? [false, true] : [false]) {
				let assets = builds[0];
				const sandbox = vm.createContext({ self: {}, console, Promise, setTimeout, clearTimeout });
				if (initial) vm.runInContext(assets["shared.js"], sandbox);
				vm.runInContext(assets["a.js"], sandbox);
				const { req, hot } = sandbox;
				const requests = [];
				req.l = (url, done) => {
					const name = path.basename(url.split("?")[0]);
					requests.push(name);
					vm.runInContext(assets[name], sandbox);
					done({ type: "load" });
				};
				if (!initial) await sandbox.loadPage();
				if (otherLoaded) await sandbox.loadOther();
				assert.equal(sandbox.pageValue, supplement ? 0 : 2);
				const oldModule = req.c["./dep.js"];
				let disposed = 0;
				oldModule.hot.dispose(() => disposed++);
				for (let step = 1; step < builds.length; step++) {
					assets = builds[step];
					req.hmrM = async () => JSON.parse(assets[`a.${req.h()}.hot-update.json`]);
					if (step === 1) {
						const manifest = await req.hmrM();
						assert(manifest.r.includes(supplement ? "old" : "shared"));
						assert(manifest.m.includes("./dep.js"));
						assert(manifest.c.includes(supplement ? "shared" : "other"), mode);
						if (supplement) assert.deepEqual(manifest.load.entries.a, ["shared"]);
					}
					await hot.check(true);
					assert.equal(hot.status(), "idle");
					if (!otherLoaded && !supplement) {
						assert.equal(req.c["./dep.js"], undefined);
						assert.equal(disposed, 1);
						assert(!requests.some(name => name.startsWith("other.")));
					} else if (!initial && step === 1) {
						assert.equal(req.c["./dep.js"], oldModule);
						assert.equal(disposed, 0);
						if (supplement) assert.equal(req.hmrS_jsonp.shared, 0);
					} else {
						assert.equal(sandbox.req("./dep.js"), 3, `${mode}, step ${step}, other loaded: ${otherLoaded}`);
						assert.equal(disposed, 1);
					}
				}
				if (!otherLoaded) assert.equal((await sandbox.loadOther()).default, 3);
			}
		}
	}
};
