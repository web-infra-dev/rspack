import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const coreRequire = createRequire(require.resolve("@rspack/core"));
const { NativeWatcher } = coreRequire("@rspack/binding");
const Watchpack = require(
	path.join(
		path.dirname(require.resolve("@rspack/core/package.json")),
		"compiled/watchpack",
	),
);

async function waitFor(predicate) {
	const deadline = Date.now() + 5000;
	while (!predicate() && Date.now() < deadline) await sleep(10);
	expect(predicate()).toBe(true);
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default ["files", "missing"].map((dependency) => ({
	name: dependency,
	description: `should match Watchpack's immediate ${dependency} notifications for a directory moved into place`,
	async run() {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-moved-"),
		);
		const watched = path.join(root, "watched");
		const staging = path.join(root, "staging");
		const destination = path.join(watched, "new");
		const ready = path.join(watched, "ready.js");
		const names = ["a.js", "nested/b.js"];
		const children = names.map((name) => path.join(destination, name));
		const ignored = path.join(destination, "ignored.js");
		const past = new Date(Date.now() - 10_000);
		fs.mkdirSync(watched);
		fs.mkdirSync(path.join(staging, "nested"), { recursive: true });
		fs.writeFileSync(ready, "0");
		fs.utimesSync(ready, past, past);
		for (const name of [...names, "ignored.js", "unregistered.js"]) {
			const file = path.join(staging, name);
			fs.writeFileSync(file, "0");
			fs.utimesSync(file, past, past);
		}
		const files = [
			ready,
			...(dependency === "files" ? [...children, ignored] : []),
		];
		const missing = dependency === "missing" ? [...children, ignored] : [];
		const options = {
			aggregateTimeout: 50,
			ignored: ignored.replaceAll("\\", "/"),
		};
		const native = new NativeWatcher(options);
		const watchpack = new Watchpack(options);
		const nativeEvents = [];
		const watchpackEvents = [];
		const nativeBatches = [];
		const watchpackBatches = [];
		const changes = (events) =>
			new Set(
				events
					.filter((event) => event.kind === "change")
					.map((event) => event.path),
			);
		native.watch(
			[files, []],
			[[], []],
			[missing, []],
			BigInt(Date.now()),
			(error, batch) => {
				expect(error).toBe(null);
				nativeBatches.push({
					changes: batch.changedFiles,
					removals: batch.removedFiles,
					raw: changes(nativeEvents),
				});
			},
			(event) => nativeEvents.push(event),
		);
		watchpack.on("change", (file) =>
			watchpackEvents.push({ kind: "change", path: file }),
		);
		watchpack.on("remove", (file) =>
			watchpackEvents.push({ kind: "remove", path: file }),
		);
		watchpack.on("aggregated", (changed, removed) =>
			watchpackBatches.push({
				changes: [...changed],
				removals: [...removed],
				raw: changes(watchpackEvents),
			}),
		);
		watchpack.watch({ files, directories: [], missing, startTime: Date.now() });
		try {
			await waitFor(
				() =>
					native
						.collectTimeInfoEntries()
						.fileTimestamps.entries.some((row) => row.path === ready) &&
					watchpack.getTimeInfoEntries().has(ready),
			);
			fs.writeFileSync(ready, "1");
			await waitFor(
				() =>
					nativeBatches.some((batch) => batch.changes.includes(ready)) &&
					watchpackBatches.some((batch) => batch.changes.includes(ready)),
			);
			nativeEvents.length = watchpackEvents.length = 0;
			nativeBatches.length = watchpackBatches.length = 0;
			fs.renameSync(staging, destination);
			await waitFor(() =>
				[nativeBatches, watchpackBatches].every((batches) =>
					children.every((child) =>
						batches.some((batch) => batch.changes.includes(child)),
					),
				),
			);
			for (const [events, batches] of [
				[watchpackEvents, watchpackBatches],
				[nativeEvents, nativeBatches],
			]) {
				expect(
					[...new Set(batches.flatMap((batch) => batch.changes))].sort(),
				).toEqual([...children].sort());
				expect(batches.flatMap((batch) => batch.removals)).toEqual([]);
				expect([...changes(events)].sort()).toEqual([...children].sort());
				for (const batch of batches) {
					for (const child of children) {
						if (batch.changes.includes(child))
							expect(batch.raw.has(child)).toBe(true);
					}
				}
			}
		} finally {
			watchpack.close();
			await native.close();
			fs.rmSync(root, { recursive: true, force: true });
		}
	},
}));
