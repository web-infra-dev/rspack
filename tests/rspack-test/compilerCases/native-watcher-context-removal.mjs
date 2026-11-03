import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { rspack } = require("@rspack/core");
const Watchpack = require(
	path.join(
		path.dirname(require.resolve("@rspack/core/package.json")),
		"compiled/watchpack",
	),
);
const deps = (paths) =>
	Object.assign([...paths], { added: paths, removed: [] });

async function waitFor(predicate) {
	const deadline = Date.now() + 5000;
	while (!predicate() && Date.now() < deadline) await sleep(10);
	expect(predicate()).toBe(true);
}

const scenarios = [
	{ name: "context-file" },
	{ name: "file-and-context", dependency: "file" },
	{ name: "missing-and-context", dependency: "missing" },
	{ name: "context-directory", directory: true },
	{ name: "directory-file-and-context", dependency: "file", directory: true },
];

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default scenarios.map((scenario) => ({
	name: scenario.name,
	description: `should match Watchpack's ${scenario.name} removal events and null change time`,
	async run() {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-context-remove-"),
		);
		const source = path.join(root, "src");
		const child = path.join(source, scenario.directory ? "child" : "child.js");
		const ready = path.join(source, "ready.js");
		const ignored = path.join(source, "ignored.js");
		fs.mkdirSync(source);
		if (scenario.directory) fs.mkdirSync(child);
		else fs.writeFileSync(child, "0");
		fs.writeFileSync(ready, "0");
		fs.writeFileSync(ignored, "0");
		const past = new Date(Date.now() - 10_000);
		for (const file of [ready, child, ignored, source])
			fs.utimesSync(file, past, past);
		const files = [
			ready,
			ignored,
			...(scenario.dependency === "file" ? [child] : []),
		];
		const directories = [source];
		const missing = scenario.dependency === "missing" ? [child] : [];
		const options = {
			aggregateTimeout: 50,
			ignored: ignored.replaceAll("\\", "/"),
		};
		await sleep(2100);
		const compiler = rspack({
			context: source,
			mode: "development",
			experiments: { nativeWatcher: true },
		});
		const native = compiler.watchFileSystem;
		const watchpack = new Watchpack(options);
		const nativeEvents = [],
			shimEvents = [],
			watchpackEvents = [];
		const nativeBatches = [],
			watchpackBatches = [];
		const listen = (watcher, events) => {
			watcher.on("change", (file, mtime) =>
				events.push({ kind: "change", file, mtime }),
			);
			watcher.on("remove", (file) => events.push({ kind: "remove", file }));
		};
		listen(native, nativeEvents);
		const startTime = Date.now();
		const handle = native.watch(
			deps(files),
			deps(directories),
			deps(missing),
			startTime,
			options,
			(error, _files, _directories, changes, removals) => {
				expect(error).toBe(null);
				nativeBatches.push({
					changes: [...changes],
					removals: [...removals],
					raw: [...nativeEvents],
				});
			},
			() => {},
		);
		listen(native.watcher, shimEvents);
		listen(watchpack, watchpackEvents);
		watchpack.on("aggregated", (changes, removals) =>
			watchpackBatches.push({
				changes: [...changes],
				removals: [...removals],
				raw: [...watchpackEvents],
			}),
		);
		watchpack.watch({ files, directories, missing, startTime });
		try {
			await waitFor(
				() =>
					native.getTimeInfoEntries().has(child) &&
					watchpack.getTimeInfoEntries().has(child),
			);
			await sleep(100);
			nativeEvents.length = shimEvents.length = watchpackEvents.length = 0;
			nativeBatches.length = watchpackBatches.length = 0;
			if (scenario.directory) fs.rmdirSync(child);
			else fs.unlinkSync(child);
			fs.unlinkSync(ignored);
			await waitFor(
				() =>
					nativeBatches.some((batch) => batch.changes.includes(source)) &&
					watchpackBatches.some((batch) => batch.changes.includes(source)),
			);
			const expectedEvents = [
				...(scenario.dependency ? [{ kind: "remove", file: child }] : []),
				{ kind: "change", file: child, mtime: null },
			];
			const normalize = (events) =>
				[...new Set(events.map((event) => JSON.stringify(event)))].sort();
			for (const events of [watchpackEvents, nativeEvents, shimEvents]) {
				expect(normalize(events)).toEqual(normalize(expectedEvents));
			}
			for (const batches of [watchpackBatches, nativeBatches]) {
				expect([...new Set(batches.flatMap((batch) => batch.changes))]).toEqual(
					[source],
				);
				expect([
					...new Set(batches.flatMap((batch) => batch.removals)),
				]).toEqual(scenario.dependency ? [child] : []);
				for (const batch of batches) {
					if (batch.changes.includes(source))
						expect(normalize(batch.raw)).toEqual(normalize(expectedEvents));
				}
			}
		} finally {
			watchpack.close();
			handle.close();
			await new Promise((resolve, reject) =>
				compiler.close((error) => (error ? reject(error) : resolve())),
			);
			fs.rmSync(root, { recursive: true, force: true });
		}
	},
}));
