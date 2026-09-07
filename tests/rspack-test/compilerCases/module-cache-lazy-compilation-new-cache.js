const fs = require("node:fs");
const path = require("node:path");
const rspack = require("@rspack/core");
const { close, write } = require("./_module-cache");

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should validate cached proxy activation and client state",
	options(context) {
		return { context: context.getSource(), entry: "./a" };
	},
	async build(context) {
		for (const persistent of [false, true]) {
			const root = context.getDist(persistent ? "persistent" : "memory");
			fs.rmSync(root, { recursive: true, force: true });
			fs.mkdirSync(root, { recursive: true });
			write(root, "index.js", 'module.exports = () => import("./target");');
			write(root, "target.js", 'module.exports = "target";');
			for (const name of ["client-a.js", "client-b.js"]) {
				write(root, name, "module.exports = () => ({ activate() {} });");
			}
			const active = new Set();
			const built = [];
			const create = (client = "client-a.js") =>
				rspack({
					context: root,
					entry: "./index.js",
					target: "node",
					mode: "development",
					devtool: false,
					incremental: false,
					cache: persistent
						? {
								type: "persistent",
								storage: {
									type: "filesystem",
									directory: path.join(root, "cache"),
								},
							}
						: { type: "memory" },
					experiments: {
						newCache: {
							module: true,
							loader: false,
							codeGeneration: false,
							devtool: false,
							minimize: false,
						},
					},
					output: { path: path.join(root, "dist") },
					plugins: [
						{
							apply(compiler) {
								// Exercise the native backend deterministically without an HTTP server.
								compiler.__internal__registerBuiltinPlugin({
									name: "LazyCompilationPlugin",
									canInherentFromParent: false,
									options: {
										currentActiveModules: () => active,
										entries: false,
										imports: true,
										client: path.join(root, client),
										reservedExternals: [],
										test: /target\.js$/,
									},
								});
								compiler.hooks.compilation.tap(
									"LazyModuleCacheTest",
									(compilation) => {
										compilation.hooks.buildModule.tap(
											"LazyModuleCacheTest",
											(module) => {
												built.push(module.identifier());
											},
										);
									},
								);
							},
						},
					],
				});
			let compiler = create();
			const run = () =>
				new Promise((resolve, reject) => {
					built.length = 0;
					compiler.run((error, stats) => {
						if (error) return reject(error);
						if (stats.hasErrors())
							return reject(
								new Error(stats.toString({ all: false, errors: true })),
							);
						resolve(
							[...stats.compilation.modules].map((module) =>
								module.identifier(),
							),
						);
					});
				});
			try {
				const first = await run();
				const proxy = first.find((id) =>
					id.startsWith("lazy-compilation-proxy|"),
				);
				expect(proxy).toBeDefined();
				expect(first).not.toContain(path.join(root, "target.js"));
				await run();
				expect(built).toEqual([]);

				active.add(proxy);
				expect(await run()).toContain(path.join(root, "target.js"));
				expect(built).toContain(proxy);
				await run();
				expect(built).toEqual([]);

				active.clear();
				expect(await run()).not.toContain(path.join(root, "target.js"));
				expect(built).toContain(proxy);

				if (persistent) {
					await close(compiler);
					compiler = create();
					await run();
					expect(built).toEqual([]);
					await close(compiler);
					compiler = create("client-b.js");
					const modules = await run();
					expect(built).toContain(proxy);
					expect(modules).toContain(path.join(root, "client-b.js"));
					expect(modules).not.toContain(path.join(root, "client-a.js"));
				}
			} finally {
				await close(compiler);
			}
		}
	},
};
