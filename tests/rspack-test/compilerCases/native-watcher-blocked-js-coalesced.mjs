import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

const CASE_NAME = 'native-watcher-blocked-js-coalesced';
const require = createRequire(import.meta.url);
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const block = (ms) =>
	Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: 'should compile queued native watcher batches only once',
	options(context) {
		const directory = context.getDist(CASE_NAME);
		fs.rmSync(directory, { recursive: true, force: true });
		fs.mkdirSync(path.join(directory, 'src'), { recursive: true });
		const source = fs.realpathSync(path.join(directory, 'src'));
		for (const name of ['a', 'b', 'c']) {
			fs.writeFileSync(
				path.join(source, `${name}.js`),
				`module.exports = "${name}-old";`,
			);
		}
		fs.writeFileSync(
			path.join(source, 'index.js'),
			'module.exports = [require("./a"), require("./b"), require("./c")];',
		);
		// Avoid startup events from the file timestamp accuracy window.
		const past = new Date(Date.now() - 10_000);
		for (const name of ['index', 'a', 'b', 'c']) {
			fs.utimesSync(path.join(source, `${name}.js`), past, past);
		}
		context.setValue('source', source);
		return {
			context: source,
			mode: 'development',
			devtool: false,
			target: 'node',
			entry: './index.js',
			output: {
				path: path.join(directory, 'dist'),
				filename: 'bundle.cjs',
				library: { type: 'commonjs2' },
			},
			experiments: { nativeWatcher: true },
		};
	},
	compiler(_context, compiler) {
		compiler.outputFileSystem = fs;
	},
	async build(context, compiler) {
		let compilations = 0;
		let builds = 0;
		let failure;
		let values;
		let lastActivity = Date.now();
		const bundle = path.join(compiler.options.output.path, 'bundle.cjs');
		compiler.hooks.afterCompile.tap(CASE_NAME, () => {
			compilations++;
			lastActivity = Date.now();
		});
		const watching = compiler.watch({ aggregateTimeout: 200 }, (error, stats) => {
			lastActivity = Date.now();
			if (error || stats.hasErrors()) {
				failure = error || new Error(stats.toString());
				return;
			}
			try {
				delete require.cache[require.resolve(bundle)];
				values = require(bundle);
				builds++;
			} catch (error) {
				failure = error;
			}
		});
		const until = async (predicate) => {
			const deadline = Date.now() + 15_000;
			while (!predicate()) {
				if (failure) throw failure;
				if (Date.now() > deadline) throw new Error('Watch did not settle');
				await wait(25);
			}
			if (failure) throw failure;
		};
		const quiet = () => !watching.running && Date.now() - lastActivity >= 3000;
		try {
			await until(() => builds > 0 && quiet());
			const initialCompilations = compilations;
			const initialBuilds = builds;
			// Each write has its own aggregation window while JS cannot drain callbacks.
			for (const [index, name] of ['a', 'b', 'c'].entries()) {
				fs.writeFileSync(
					path.join(context.getValue('source'), `${name}.js`),
					`module.exports = "${name}-new";`,
				);
				// The final block must exceed OS event latency plus aggregateTimeout.
				block(index === 2 ? 2500 : 500);
			}
			lastActivity = Date.now();
			await until(() => builds > initialBuilds && quiet());
			context.setValue('compilations', compilations - initialCompilations);
			context.setValue('values', values);
		} finally {
			await new Promise((resolve, reject) =>
				watching.close((error) => (error ? reject(error) : resolve())),
			);
		}
	},
	async check({ context }) {
		expect(context.getValue('values')).toEqual(['a-new', 'b-new', 'c-new']);
		expect(context.getValue('compilations')).toBe(1);
	},
};
