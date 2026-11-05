import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [false, true].flatMap(nativeWatcher =>
	[false, true].map(preserveMtime => {
		const name = `rename-replacement-${nativeWatcher ? "native" : "watchpack"}-${preserveMtime ? "same-mtime" : "new-mtime"}`;
		return {
			name: `${nativeWatcher ? "native" : "watchpack"}-${preserveMtime ? "same-mtime" : "new-mtime"}`,
			description: preserveMtime
				? "should ignore an immediate replacement with unchanged mtime like watchpack"
				: "should compile the replacement after renaming the original to a backup",
			options(context) {
				const directory = context.getDist(name);
				fs.rmSync(directory, { recursive: true, force: true });
				fs.mkdirSync(path.join(directory, "src"), { recursive: true });
				const source = fs.realpathSync(path.join(directory, "src"));
				const file = path.join(source, "index.js");
				const staged = path.join(directory, "replacement.js");
				fs.writeFileSync(file, 'module.exports = "old";');
				fs.writeFileSync(staged, 'module.exports = "new";');
				const past = new Date(Date.now() - 10_000);
				fs.utimesSync(file, past, past);
				if (preserveMtime) fs.utimesSync(staged, past, past);
				context.setValue("file", file);
				context.setValue("staged", staged);
				return {
					context: source,
					mode: "development",
					devtool: false,
					target: "node",
					entry: "./index.js",
					output: {
						path: path.join(directory, "dist"),
						filename: "bundle.cjs",
						library: { type: "commonjs2" }
					},
					experiments: { nativeWatcher }
				};
			},
			compiler(_context, compiler) {
				compiler.outputFileSystem = fs;
			},
			async build(context, compiler) {
				let builds = 0;
				let failure;
				let value;
				const bundle = path.join(compiler.options.output.path, "bundle.cjs");
				const watching = compiler.watch(
					// Run native's ignored check on JS before it stats either rename event.
					{ aggregateTimeout: 300, ignored: () => false },
					(error, stats) => {
						if (error || stats.hasErrors()) {
							failure = error || new Error(stats.toString());
							return;
						}
						try {
							delete require.cache[require.resolve(bundle)];
							value = require(bundle);
							builds++;
						} catch (error) {
							failure = error;
						}
					}
				);
				const until = async predicate => {
					const deadline = Date.now() + 10_000;
					while (!predicate()) {
						if (failure) throw failure;
						if (Date.now() > deadline)
							throw new Error("Timed out waiting for rename rebuild");
						await wait(25);
					}
					if (failure) throw failure;
				};
				try {
					await until(() => builds > 0 && !watching.running);
					expect(value).toBe("old");
					// watch() resumes after the compiler callback.
					await wait(600);
					const initialBuilds = builds;
					const file = context.getValue("file");
					fs.renameSync(file, `${file}.backup`);
					fs.renameSync(context.getValue("staged"), file);
					if (preserveMtime) {
						await wait(1200);
						if (failure) throw failure;
						expect(builds).toBe(initialBuilds);
						expect(value).toBe("old");
					} else {
						await until(() => builds > initialBuilds && !watching.running);
						expect(value).toBe("new");
						expect(compiler.modifiedFiles.has(file)).toBe(true);
						expect(compiler.removedFiles.has(file)).toBe(false);
					}
				} finally {
					await new Promise((resolve, reject) =>
						watching.close(error => (error ? reject(error) : resolve()))
					);
				}
			}
		};
	})
);
