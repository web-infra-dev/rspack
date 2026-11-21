import path from "node:path";
import { promisify } from "node:util";
import { runInNewContext } from "node:vm";
import { rspack, RuntimeGlobals } from "@rspack/core";

const webpackPublicPath = "__webpack_require__.p";

function createPlugin() {
	let started = 0;
	const callbacks = [];
	let startBoth;
	const bothStarted = new Promise(resolve => {
		startBoth = resolve;
	});

	return {
		apply(compiler) {
			const { RuntimeGlobals: scoped, RuntimeModule } = compiler.rspack;
			const checkScope = () => {
				expect(RuntimeGlobals.publicPath).toBe(scoped.publicPath);
				expect(rspack.webpack.RuntimeGlobals.require).toBe(scoped.require);
			};
			checkScope();

			const { beforeRun } = compiler.hooks;
			const registeredTaps = new Set();
			beforeRun.intercept({
				register(tap) {
					registeredTaps.add(tap);
					return tap;
				},
				tap(tap) {
					expect(registeredTaps.has(tap)).toBe(true);
					checkScope();
				}
			});
			beforeRun.tapPromise("PublicPathPlugin", async () => {
				const publicPath = RuntimeGlobals.publicPath;
				if (++started === 2) startBoth();
				await bothStarted;
				expect(publicPath).toBe(scoped.publicPath);
				expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);
			});
			beforeRun.tapAsync("PublicPathPlugin", (_compiler, callback) => {
				const publicPath = RuntimeGlobals.publicPath;
				callbacks.push(() => {
					try {
						expect(publicPath).toBe(scoped.publicPath);
					} catch (error) {
						return callback(error);
					}
					callback();
				});
				if (callbacks.length === 2) {
					for (const resume of callbacks) resume();
				}
			});
			beforeRun.tap("PublicPathPlugin", checkScope);

			compiler.hooks.thisCompilation.tap("PublicPathPlugin", compilation => {
				checkScope();
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"PublicPathPlugin",
					(_chunk, requirements) => {
						checkScope();
						requirements.add(RuntimeGlobals.publicPath);
					}
				);
				compilation.hooks.runtimeRequirementInTree
					.for(RuntimeGlobals.publicPath)
					.tap("PublicPathPlugin", (chunk, requirements) => {
						checkScope();
						expect(requirements.has(RuntimeGlobals.publicPath)).toBe(true);
						const mod = new RuntimeModule(
							"plugin public path",
							RuntimeModule.STAGE_TRIGGER
						);
						mod.generate = () =>
							'__webpack_require__.p += "plugin/"; globalThis.pluginPath = __webpack_require__.p;';
						compilation.addRuntimeModule(chunk, mod);
					});

				compilation.hooks.processAssets.tapAsync(
					"PublicPathPlugin",
					(_assets, callback) => {
						const publicPath = RuntimeGlobals.publicPath;
						setImmediate(() => {
							try {
								expect(publicPath).toBe(scoped.publicPath);
								expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);
							} catch (error) {
								return callback(error);
							}
							callback();
						});
					}
				);
			});
		}
	};
}

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
export default {
	description: "a shared plugin reads runtime globals in synchronous compiler scopes",
	options(context) {
		const plugin = createPlugin();
		return ["webpack", "rspack"].map(runtimeMode => ({
			name: runtimeMode,
			mode: "development",
			target: "node",
			devtool: false,
			entry: "data:text/javascript,export default 1",
			experiments: { runtimeMode },
			output: {
				path: context.getDist(),
				filename: runtimeMode + ".js",
				publicPath: "/" + runtimeMode + "/"
			},
			plugins: [plugin]
		}));
	},
	async build(context, compiler) {
		try {
			const [first, second] = compiler.compilers;
			const rspackPublicPath = second.rspack.RuntimeGlobals.publicPath;
			expect(first.rspack.RuntimeGlobals.publicPath).toBe(webpackPublicPath);
			expect(rspackPublicPath).not.toBe(webpackPublicPath);
			expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);

			let detachedRead;
			let nestedCompletion;
			first.hooks.shutdown.tapAsync("NestedCompiler", callback => {
				setImmediate(callback);
			});
			first.hooks.initialize.tap("NestedCompiler", () => {
				expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);
				throw new Error("nested compiler error");
			});
			second.hooks.initialize.tap("NestedCompiler", () => {
				expect(RuntimeGlobals.publicPath).toBe(rspackPublicPath);
				expect(() => first.hooks.initialize.call()).toThrow("nested compiler error");
				expect(RuntimeGlobals.publicPath).toBe(rspackPublicPath);
				detachedRead = Promise.resolve().then(() => RuntimeGlobals.publicPath);
				nestedCompletion = new Promise((resolve, reject) => {
					first.hooks.shutdown.callAsync(error => {
						if (error) return reject(error);
						resolve(RuntimeGlobals.publicPath);
					});
				});
			});
			second.hooks.initialize.callAsync(error => {
				expect(error).toBeNull();
				expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);
			});
			expect(await detachedRead).toBe(webpackPublicPath);
			expect(await nestedCompletion).toBe(webpackPublicPath);

			const stats = await promisify(compiler.run.bind(compiler))();
			expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
			expect(RuntimeGlobals.publicPath).toBe(webpackPublicPath);
			for (const child of compiler.compilers) {
				const file = path.join(context.getDist(), child.name + ".js");
				const scope = {};
				runInNewContext(child.outputFileSystem.readFileSync(file, "utf8"), scope);
				expect(scope.pluginPath).toBe("/" + child.name + "/plugin/");
			}
		} finally {
			await promisify(compiler.close.bind(compiler))();
		}
	}
};
