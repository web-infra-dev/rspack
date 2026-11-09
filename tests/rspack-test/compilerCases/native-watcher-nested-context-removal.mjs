import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { rspack } = require("@rspack/core");
const deps = (paths) =>
	Object.assign([...paths], { added: paths, removed: [] });

async function waitFor(predicate) {
	const deadline = Date.now() + 5000;
	while (!predicate() && Date.now() < deadline) await sleep(10);
	expect(predicate()).toBe(true);
}

const scenarios = [
	{ name: "context-only" },
	{ name: "populated-context", populated: true },
	{ name: "populated-file", populated: true, insideDependency: "file" },
	{ name: "populated-missing", populated: true, insideDependency: "missing" },
	{ name: "also-file", dependency: "file" },
	{ name: "also-missing", dependency: "missing" },
];

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default scenarios.map((scenario) => ({
	name: scenario.name,
	description: `should report removal events for a nested context ${scenario.name}`,
	async run() {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-nested-context-remove-"),
		);
		const source = path.join(root, "src");
		const child = path.join(source, "child");
		const inside = path.join(child, "inside.js");
		const ready = path.join(source, "ready.js");
		const scanned = path.join(source, "scanned.js");
		const ignored = path.join(source, "ignored.js");
		fs.mkdirSync(source);
		fs.mkdirSync(child);
		if (scenario.populated) fs.writeFileSync(inside, "0");
		fs.writeFileSync(ready, "0");
		fs.writeFileSync(scanned, "0");
		fs.writeFileSync(ignored, "0");
		const past = new Date(Date.now() - 10_000);
		for (const file of [
			ready,
			scanned,
			child,
			ignored,
			source,
			...(scenario.populated ? [inside] : []),
		])
			fs.utimesSync(file, past, past);
		const files = [
			ready,
			ignored,
			...(scenario.dependency === "file" ? [child] : []),
			...(scenario.insideDependency === "file" ? [inside] : []),
		];
		const directories = [source, child];
		const missing = [
			...(scenario.dependency === "missing" ? [child] : []),
			...(scenario.insideDependency === "missing" ? [inside] : []),
		];
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
		const nativeEvents = [],
			shimEvents = [];
		const nativeBatches = [];
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
		try {
			// The unregistered file gets time info from the context scan.
			await waitFor(() => native.getTimeInfoEntries().has(scanned));
			nativeEvents.length = shimEvents.length = 0;
			nativeBatches.length = 0;
			if (scenario.populated) fs.rmSync(child, { recursive: true });
			else fs.rmdirSync(child);
			fs.unlinkSync(ignored);
			await waitFor(() =>
				nativeBatches.some((batch) => batch.changes.includes(source)),
			);
			const expectedEvents = [
				...(scenario.dependency ? [{ kind: "remove", file: child }] : []),
				...(scenario.insideDependency
					? [{ kind: "remove", file: inside }]
					: []),
				{ kind: "change", file: child, mtime: null },
				...(scenario.populated
					? [{ kind: "change", file: inside, mtime: null }]
					: []),
			];
			// Recursive removal can change the directory before deleting it.
			const normalize = (events) =>
				[
					...new Set(
						events
							.filter(
								(event) =>
									!(
										scenario.populated &&
										event.kind === "change" &&
										event.file === child &&
										typeof event.mtime === "number"
									),
							)
							.map((event) => JSON.stringify(event)),
					),
				].sort();
			for (const events of [nativeEvents, shimEvents]) {
				expect(normalize(events)).toEqual(normalize(expectedEvents));
			}
			expect(
				[...new Set(nativeBatches.flatMap((batch) => batch.changes))].sort(),
			).toEqual([source, ...(scenario.populated ? [child] : [])].sort());
			expect([
				...new Set(nativeBatches.flatMap((batch) => batch.removals)),
			]).toEqual([
				...(scenario.dependency ? [child] : []),
				...(scenario.insideDependency ? [inside] : []),
			]);
			for (const batch of nativeBatches) {
				if (batch.changes.includes(source))
					expect(normalize(batch.raw)).toEqual(normalize(expectedEvents));
			}
		} finally {
			handle.close();
			await new Promise((resolve, reject) =>
				compiler.close((error) => (error ? reject(error) : resolve())),
			);
			fs.rmSync(root, { recursive: true, force: true });
		}
	},
}));
