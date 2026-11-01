import path from "node:path";
import { createFsFromVolume, Volume } from "memfs";

const PLUGIN_NAME = "DevServerLikePlugin";

const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

// Taps hooks the way @rspack/dev-server's `apply()` does, with no branching on
// the compiler type, and records what each hook did.
class DevServerLikePlugin {
	fs = createFsFromVolume(new Volume());
	applied = 0;
	setupCalls = 0;
	stopCalls = 0;
	invalidCalls = 0;
	doneCalls = 0;
	hasLogger = false;
	stopFinished = false;

	apply(compiler) {
		this.applied++;
		this.hasLogger =
			typeof compiler.getInfrastructureLogger(PLUGIN_NAME).info === "function";
		let setupPromise;
		compiler.hooks.watchRun.tapPromise(PLUGIN_NAME, () => {
			// Asynchronous like the dev server's setup, which swaps the output fs.
			setupPromise ??= delay(10).then(() => {
				this.setupCalls++;
				compiler.outputFileSystem = this.fs;
			});
			return setupPromise;
		});
		compiler.hooks.invalid.tap(PLUGIN_NAME, () => {
			this.invalidCalls++;
		});
		compiler.hooks.done.tap(PLUGIN_NAME, () => {
			this.doneCalls++;
		});
		let stopped = false;
		compiler.hooks.shutdown.tapPromise(PLUGIN_NAME, async () => {
			if (stopped) return;
			stopped = true;
			this.stopCalls++;
			await delay(10);
			this.stopFinished = true;
		});
	}
}

const close = compiler =>
	new Promise((resolve, reject) => {
		compiler.close(err => (err ? reject(err) : resolve()));
	});

const createOptions = plugins => {
	const context = path.join(import.meta.dirname, "../fixtures");
	return {
		0: {
			name: "a",
			mode: "development",
			context,
			entry: "./a.js",
			output: { filename: "a.js" }
		},
		1: {
			name: "b",
			mode: "development",
			context,
			entry: "./b.js",
			output: { filename: "b.js" }
		},
		plugins
	};
};

const bundlePaths = compiler =>
	compiler.compilers.map(child =>
		path.join(child.outputPath, child.options.output.filename)
	);

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
export default [
	(() => {
		const applied = [];
		const pluginFunction = function (compiler) {
			applied.push(["function", this, compiler]);
		};
		const plugins = [
			{
				apply(compiler) {
					applied.push(["instance", compiler]);
				}
			},
			pluginFunction,
			false,
			null,
			undefined,
			0,
			""
		];
		return {
			description: "should apply plugins once to the MultiCompiler",
			options: () => createOptions(plugins),
			async compiler(context, compiler) {
				expect(applied).toEqual([
					["instance", compiler],
					["function", compiler, compiler]
				]);
				expect(compiler.options.plugins).toBe(plugins);
			},
			build: async (context, compiler) => close(compiler)
		};
	})(),
	(() => {
		const plugin = new DevServerLikePlugin();
		const originalFs = createFsFromVolume(new Volume());
		return {
			description:
				"should await watchRun before emitting and shutdown when closing",
			options: () => createOptions([plugin]),
			async compiler(context, compiler) {
				expect(plugin.applied).toBe(1);
				expect(plugin.hasLogger).toBe(true);
				compiler.outputFileSystem = originalFs;
			},
			async build(context, compiler) {
				await new Promise((resolve, reject) => {
					let handled = 0;
					compiler.watch({}, err => {
						if (err) return reject(err);
						if (++handled === 1) resolve();
					});
				});

				expect(plugin.setupCalls).toBe(1);
				expect(plugin.doneCalls).toBe(1);
				// The awaited watchRun swapped the fs before anything was emitted.
				for (const bundle of bundlePaths(compiler)) {
					expect(plugin.fs.existsSync(bundle)).toBe(true);
					expect(originalFs.existsSync(bundle)).toBe(false);
				}

				await new Promise(resolve => {
					compiler.watching.invalidate(resolve);
				});

				// `invalid` is a MultiHook, so the tap fires once per child.
				expect(plugin.invalidCalls).toBe(2);
				expect(plugin.doneCalls).toBe(2);
				expect(plugin.setupCalls).toBe(1);

				await close(compiler);
				expect(plugin.stopCalls).toBe(1);
				expect(plugin.stopFinished).toBe(true);
			}
		};
	})(),
	(() => {
		const plugin = new DevServerLikePlugin();
		const originalFs = createFsFromVolume(new Volume());
		return {
			description: "should stay passive for a one-shot run",
			options: () => createOptions([plugin]),
			async compiler(context, compiler) {
				compiler.outputFileSystem = originalFs;
			},
			async build(context, compiler) {
				await new Promise((resolve, reject) => {
					compiler.run(err => (err ? reject(err) : resolve()));
				});

				expect(plugin.doneCalls).toBe(1);
				expect(plugin.setupCalls).toBe(0);
				for (const bundle of bundlePaths(compiler)) {
					expect(originalFs.existsSync(bundle)).toBe(true);
				}

				await close(compiler);
				expect(plugin.stopCalls).toBe(1);
				expect(plugin.stopFinished).toBe(true);
			}
		};
	})()
];
