const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const { rspack } = require("@rspack/core");

describe("NodeWatchFileSystem getInfo", () => {
	it("keeps an active aggregate callback and consumes paused changes once", async () => {
		const root = await fs.mkdtemp(path.join(os.tmpdir(), "rspack-watchpack-"));
		const entry = path.join(root, "entry.js");
		await fs.writeFile(entry, "export default 1;\n");
		const beforeWatch = new Date(Date.now() - 3000);
		await fs.utimes(entry, beforeWatch, beforeWatch);

		const compiler = rspack({
			context: root,
			entry,
			experiments: { nativeWatcher: false }
		});
		const watchFileSystem = compiler.watchFileSystem;
		const callbackChanges = [];
		const watcher = watchFileSystem.watch(
			[entry],
			[],
			[],
			Date.now(),
			{ aggregateTimeout: 1000 },
			(_error, _fileTimes, _contextTimes, changes) =>
				callbackChanges.push(changes),
			() => {}
		);

		try {
			const watchpack = watchFileSystem.watcher;
			watchpack._onChange(entry, Date.now(), entry, "change");
			expect(watcher.hasPendingEvents()).toBe(true);
			expect(watcher.getInfo().changes).toEqual(new Set([entry]));
			expect(watchpack.aggregateTimer).toBeDefined();
			watchpack._onTimeout();
			expect(callbackChanges).toEqual([new Set([entry])]);

			watchpack._onChange(entry, Date.now(), entry, "change");
			expect(watcher.hasPendingEvents()).toBe(true);
			expect(watcher.getInfo().changes).toEqual(new Set([entry]));
			expect(watcher.getInfo().changes).toEqual(new Set());
			expect(watcher.hasPendingEvents()).toBe(false);
		} finally {
			watcher.close();
			await new Promise(resolve => compiler.close(resolve));
			await fs.rm(root, { force: true, recursive: true });
		}
	});
});
