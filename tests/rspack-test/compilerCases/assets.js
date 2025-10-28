const { createFsFromVolume, Volume } = require("memfs");
/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should get assets with both `getAssets` and `assets`(getter)",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							let list = compilation.getAssets();
							let map = compilation.assets;

							expect(Object.keys(map)).toHaveLength(list.length);

							list.forEach(a => {
								const b = map[a.name];
								expect(a.source.buffer()).toEqual(b.buffer());
							});
						});
					});
				}
			}]
		};
	}
}, {
	description: "should have error if the asset to be emitted is exist",
	error: true,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							const { RawSource } = require("webpack-sources");
							compilation.emitAsset(
								"main.js",
								new RawSource(`module.exports = "I'm the right main.js"`)
							);
						});
					});
				}
			}]
		};
	},
	async check({ stats }) {
		expect(stats.errors[0].message).toMatchInlineSnapshot(
			`× Conflict: Multiple assets emit different content to the same filename main.js`
		);
	}
}, (() => {
	const mockFn = rstest.fn();
	return {
		description: "should throw if the asset to be updated is not exist",
		options(context) {
			return {
				context: context.getSource(),
				entry: "./d",
				plugins: [{
					apply(compiler) {
						compiler.hooks.compilation.tap("Plugin", compilation => {
							compilation.hooks.processAssets.tap("Plugin", () => {
								const { RawSource } = require("webpack-sources");
								try {
									compilation.updateAsset(
										"something-else.js",
										new RawSource(`module.exports = "something-else"`),
										{
											minimized: true,
											development: true,
											related: {},
											hotModuleReplacement: false
										}
									);
								} catch (err) {
									mockFn();
									expect(err.toString()).toMatchInlineSnapshot(
										`Error: Called Compilation.updateAsset for not existing filename something-else.js`
									);
								}
							});
						});
					}
				}]
			};
		},
		async check() {
			expect(mockFn).toHaveBeenCalled();
		}
	};
})(), {
	description: "should emit assets correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						let assets = compilation.getAssets();
						expect(assets.length).toBe(0);
						const { RawSource } = require("webpack-sources");
						compilation.emitAsset(
							"dd.js",
							new RawSource(`module.exports = "This is dd"`)
						);
						compilation.hooks.processAssets.tap("Plugin", assets => {
							let names = Object.keys(assets);

							expect(names.length).toBe(2); // ["main.js", "dd.js"]
							expect(names.includes("main.js")).toBeTruthy();
							expect(assets["main.js"].source().includes("This is d"));

							expect(names.includes("dd.js")).toBeTruthy();
						});
					});
				}
			}]
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = createFsFromVolume(new Volume());
	},
	async check({ compiler }) {
		if (
			compiler.outputFileSystem.existsSync("/directory/main.js") &&
			compiler.outputFileSystem.existsSync("/directory/dd.js")
		) {
			const dd = compiler.outputFileSystem.readFileSync("/directory/dd.js", "utf-8");

			if (dd !== `module.exports="This is dd";`) {
				throw new Error("File content is not correct");
			}
		}
	}
}, {
	description: "should update assets",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap("Plugin", () => {
							const oldSource = compilation.assets["main.js"];
							expect(oldSource).toBeTruthy();
							expect(oldSource.source().includes("This is d")).toBeTruthy();
							const { RawSource } = require("webpack-sources");
							const updatedSource = new RawSource(
								`module.exports = "This is the updated d"`
							);
							compilation.updateAsset(
								"main.js",
								source => {
									expect(source.buffer()).toEqual(oldSource.buffer());
									return updatedSource;
								},
								_ => _
							);

							const newSource = compilation.assets["main.js"];
							expect(newSource).toBeTruthy();
							expect(newSource.buffer()).toStrictEqual(updatedSource.buffer());
						});
					});
				}
			}]
		};
	}
}, {
	description:
		"compilation.updateAsset should preserve fields in addition to KnownAssetInfo",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("Plugin", compilation => {
						compilation.hooks.processAssets.tap(
							{
								name: "Plugin",
								stage:
									compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE - 1
							},
							() => {
								compilation.getAssets().forEach(a => {
									const new_info = {
										...a.info,
										unminified_name: a.name ?? "non_empty_str"
									};
									compilation.updateAsset(a.name, a.source, new_info);
								});
							}
						);
					});

					compiler.hooks.done.tapPromise("Plugin", async stats => {
						stats.compilation.getAssets().forEach(a => {
							expect(a.info.unminified_name).toBeTruthy();
						});
					});
				}
			}]
		};
	}
}];
