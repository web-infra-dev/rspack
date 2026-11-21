import fs from "node:fs";
import path from "node:path";

const NAME = "native-watcher-context-watch-errors";
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
function write(file, content, initial = false) {
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	if (initial) {
		const past = new Date(Date.now() - 10_000);
		fs.utimesSync(file, past, past);
	}
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default ["restore", "recursive-alias"].map(name => {
	const restore = name === "restore";
	return {
		name,
		description: restore
			? "should keep watching after a context symlink target disappears"
			: "should report symlinked dependencies covered by a recursive watch",
		options(context) {
			fs.mkdirSync(context.getDist(NAME), { recursive: true });
			const source = fs.realpathSync(context.getDist(NAME));
			const files = restore
				? { "rc/real/a.txt": "a", "rc/x/seed.txt": "s" }
				: { "zr/real/f.txt": "f", "zr/real/g.txt": "g" };
			write(path.join(source, "index.js"), "module.exports = 1;", true);
			for (const [file, content] of Object.entries(files)) write(path.join(source, file), content, true);
			const target = restore ? "../real" : "real";
			const link = path.join(source, restore ? "rc/x/link" : "zr/link");
			fs.symlinkSync(process.platform === "win32" ? path.resolve(path.dirname(link), target) : target, link, "junction");
			context.setValue("source", source);
			return {
				context: source, mode: "development", devtool: false, target: "node",
				entry: "./index.js", resolve: { symlinks: false },
				output: { path: path.join(source, "dist") },
				experiments: { nativeWatcher: true },
				plugins: [compiler => compiler.hooks.thisCompilation.tap(NAME, compilation => {
					compilation.contextDependencies.add(path.join(source, restore ? "rc" : "zr/link"));
					compilation.missingDependencies.add(path.join(source, restore ? "rc/m1/m2/m.txt" : "zr/m1/m2/m.txt"));
					if (!restore) {
						compilation.fileDependencies.add(path.join(source, "zr/real/f.txt"));
						compilation.fileDependencies.add(path.join(source, "zr/link/g.txt"));
					}
				})]
			};
		},
		async build(context, compiler) {
			const source = context.getValue("source");
			let pending, timer, failure, builds = 0;
			const next = () => new Promise((resolve, reject) => {
				pending = error => {
					clearTimeout(timer);
					pending = undefined;
					error ? reject(error) : resolve();
				};
				timer = setTimeout(() => reject(new Error(`${name}: context edit did not rebuild`)), 4_000);
			});
			const first = next();
			const watching = compiler.watch({ aggregateTimeout: 50, followSymlinks: true }, (error, stats) => {
				builds++;
				failure = error || (stats.hasErrors() ? new Error(stats.toString()) : undefined);
				pending?.(failure);
			});
			const step = async change => {
				let before;
				do { before = builds; await wait(300); } while (builds !== before);
				if (failure) throw failure;
				const done = next();
				change();
				await done;
			};
			try {
				await first;
				if (restore) {
					await step(() => fs.rmSync(path.join(source, "rc/real"), { recursive: true }));
					await step(() => write(path.join(source, "rc/m1/m2/m.txt"), "m"));
					await step(() => write(path.join(source, "rc/x/new.txt"), "n"));
					await step(() => write(path.join(source, "rc/x/seed.txt"), "s2"));
					await step(() => write(path.join(source, "rc/top.txt"), "t"));
				} else {
					await step(() => write(path.join(source, "zr/real/f.txt"), "f1"));
					await step(() => write(path.join(source, "zr/real/g.txt"), "g1"));
					expect([...compiler.modifiedFiles].includes(path.join(source, "zr/link/g.txt"))).toEqual(true);
					await step(() => write(path.join(source, "zr/real/new.txt"), "n"));
				}
			} finally {
				clearTimeout(timer);
				await new Promise((resolve, reject) => watching.close(error => error ? reject(error) : resolve()));
			}
		}
	};
});
