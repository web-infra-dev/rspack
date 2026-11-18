import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
	"@rspack/binding",
);
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

const scenarios = [
	{ name: "nested-context", nested: true },
	{ name: "standalone-context" },
	{ name: "file-in-context", nested: true, file: true },
];

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default scenarios.map((scenario) => ({
	name: scenario.name,
	description: `should preserve the initial removal of a missing ${scenario.name}`,
	async run() {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-initial-missing-"),
		);
		const source = path.join(root, "src");
		const child = path.join(source, "child");
		fs.mkdirSync(source);
		const past = new Date(Date.now() - 60_000);
		fs.utimesSync(source, past, past);
		await sleep(2100);
		const files = scenario.file ? [child] : [];
		const directories = [
			...(scenario.nested ? [source] : []),
			...(scenario.file ? [] : [child]),
		];
		const options = { aggregateTimeout: 50 };
		const native = new NativeWatcher(options);
		const watchpack = new Watchpack(options);
		const nativeEvents = [],
			watchpackEvents = [],
			nativeBatches = [],
			watchpackBatches = [];
		const startTime = Date.now();
		native.watch(
			[files, []],
			[directories, []],
			[[], []],
			BigInt(startTime),
			(error, batch) => {
				expect(error).toBe(null);
				nativeBatches.push({
					changedFiles: batch.changedFiles,
					removedFiles: batch.removedFiles,
					raw: [...nativeEvents],
				});
			},
			(event) => nativeEvents.push({ kind: event.kind, path: event.path }),
		);
		watchpack.on("change", (file) =>
			watchpackEvents.push({ kind: "change", path: file }),
		);
		watchpack.on("remove", (file) =>
			watchpackEvents.push({ kind: "remove", path: file }),
		);
		watchpack.on("aggregated", (changes, removals) =>
			watchpackBatches.push({
				changedFiles: [...changes],
				removedFiles: [...removals],
				raw: [...watchpackEvents],
			}),
		);
		watchpack.watch({ files, directories, missing: [], startTime });
		try {
			await waitFor(
				() => nativeBatches.length > 0 && watchpackBatches.length > 0,
			);
			const expectedEvents = [JSON.stringify({ kind: "remove", path: child })];
			const normalize = (events) =>
				[...new Set(events.map((event) => JSON.stringify(event)))].sort();
			for (const events of [watchpackEvents, nativeEvents])
				expect(normalize(events)).toEqual(expectedEvents);
			for (const batches of [watchpackBatches, nativeBatches]) {
				expect([
					...new Set(batches.flatMap((batch) => batch.removedFiles)),
				]).toEqual([child]);
				for (const batch of batches) {
					if (batch.removedFiles.includes(child))
						expect(normalize(batch.raw)).toEqual(expectedEvents);
				}
			}
			expect(watchpackBatches.flatMap((batch) => batch.changedFiles)).toEqual(
				[],
			);
			// Native scans already invalidate the parent; preserve that aggregate.
			expect([
				...new Set(nativeBatches.flatMap((batch) => batch.changedFiles)),
			]).toEqual(scenario.nested ? [source] : []);
			expect(fs.existsSync(child)).toBe(false);
		} finally {
			watchpack.close();
			await native.close();
			fs.rmSync(root, { recursive: true, force: true });
		}
	},
}));
