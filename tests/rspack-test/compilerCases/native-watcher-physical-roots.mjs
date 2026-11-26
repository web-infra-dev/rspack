import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
	"@rspack/binding",
);

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description: "should watch both directory trees and keep a shared file subscription",
	async run() {
		const temp = fs.realpathSync(os.tmpdir());
		const secondVolume = process.env.RSPACK_WATCHER_TEST_VOLUME;
		if (secondVolume) {
			expect(path.parse(secondVolume).root.toLowerCase()).not.toBe(
				path.parse(temp).root.toLowerCase(),
			);
		}
		const left = fs.mkdtempSync(path.join(temp, "rspack-left-"));
		const right = fs.mkdtempSync(path.join(secondVolume ?? temp, "rspack-right-"));
		const roots = [left, right];
		const keep = path.join(left, "keep.js");
		const watcher = new NativeWatcher({ aggregateTimeout: 20 });
		const raw = new Set();
		const changed = new Set();
		let failure;
		const watch = (directories, missing) => watcher.watch(
			[[], []], directories, missing, BigInt(Date.now()),
			(error, batch) => {
				if (error) failure = error;
				else for (const file of batch.changedFiles) changed.add(file);
			},
			event => raw.add(event.path),
		);
		async function until(predicate) {
			const deadline = Date.now() + 10000;
			while (!failure && !predicate() && Date.now() < deadline) await sleep(10);
			if (failure) throw failure;
			expect(predicate()).toBe(true);
		}

		try {
			const seeds = roots.map(root => path.join(root, "seed.js"));
			for (const seed of seeds) {
				fs.writeFileSync(seed, "");
				fs.utimesSync(seed, 1_600_000_000, 1_600_000_000);
			}
			watch([roots, []], [[keep], []]);
			await until(() => {
				const { entries } = watcher.collectTimeInfoEntries().fileTimestamps;
				return seeds.every(seed => entries.some(row => row.path === seed));
			});
			const files = roots.map(root => path.join(root, "live.js"));
			for (const file of files) fs.writeFileSync(file, "live");
			await until(() => files.every(file => raw.has(file)) && roots.every(root => changed.has(root)));

			watch([[], roots], [[], []]);
			await until(() => watcher.collectTimeInfoEntries().directoryTimestamps.entries.length === 0);
			fs.writeFileSync(keep, "still watched");
			await until(() => raw.has(keep) && changed.has(keep));
		} finally {
			await watcher.close();
			for (const root of roots) fs.rmSync(root, { recursive: true, force: true });
		}
	},
};
