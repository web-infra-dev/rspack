import { EventEmitter, on } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setImmediate } from "node:timers/promises";

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should rebuild once for a restored missing directory with a preserved mtime",
	options() {
		return { entry: {}, experiments: { nativeWatcher: true } };
	},
	async build(_context, compiler) {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-restored-")
		);
		const src = path.join(root, "src");
		const restored = path.join(src, "restored");
		const marker = path.join(root, "marker");
		const old = 1_600_000_000.123;
		fs.mkdirSync(src);
		fs.mkdirSync(marker);
		fs.utimesSync(marker, old, old);
		compiler.hooks.afterCompile.tap("RestoreMissingDirectory", compilation => {
			compilation.contextDependencies.add(src);
			compilation.contextDependencies.add(marker);
			compilation.missingDependencies.add(restored);
			if (!fs.existsSync(restored)) {
				fs.mkdirSync(restored);
				fs.utimesSync(restored, old, old);
				fs.utimesSync(src, old, old);
			}
		});

		const events = new EventEmitter();
		const builds = on(events, "build", { signal: AbortSignal.timeout(10_000) });
		const nextBuild = async () => (await builds.next()).value[0];
		let buildCount = 0;
		const watching = compiler.watch({ aggregateTimeout: 20 }, (error, stats) => {
			if (error || stats.hasErrors()) {
				events.emit("error", error || new Error(stats.toString()));
				return;
			}
			buildCount++;
			events.emit("build", compiler.modifiedFiles);
		});
		try {
			await nextBuild();
			expect(await nextBuild()).toEqual(new Set([src, restored]));
			await setImmediate();

			// Queue an attribute-only event and a separate rebuild in order.
			const watcher = compiler.watchFileSystem;
			watcher.triggerEvent("change", restored);
			watcher.triggerEvent("change", marker);
			expect(await nextBuild()).toEqual(new Set([marker]));
			await setImmediate();

			fs.utimesSync(restored, old + 60, old + 60);
			watcher.triggerEvent("change", restored);
			expect(await nextBuild()).toEqual(new Set([src, restored]));
			expect(buildCount).toBe(4);
		} finally {
			await builds.return();
			await new Promise((resolve, reject) =>
				watching.close(error => (error ? reject(error) : resolve()))
			);
			fs.rmSync(root, { force: true, recursive: true });
		}
	}
};
