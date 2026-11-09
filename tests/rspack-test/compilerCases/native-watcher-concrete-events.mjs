import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
	"@rspack/binding",
);

async function waitFor(predicate) {
	const deadline = Date.now() + 5000;
	while (!predicate() && Date.now() < deadline) await sleep(10);
	expect(predicate()).toBe(true);
}

const scenarios = [
	{ name: "create", kind: "create", count: 1, native: true },
	{
		name: "edit-and-create",
		kind: "create",
		count: 1,
		edit: true,
		native: true,
	},
	{ name: "remove", kind: "remove", count: 1 },
	{
		name: "delayed-create-after-delete",
		kind: "create",
		count: 1,
		absent: true,
	},
	{ name: "context-burst", kind: "create", count: 128 },
	{ name: "registered-burst", kind: "change", count: 2048, registered: true },
	{ name: "ignored-and-unrelated", filtered: true, edit: true, count: 0 },
];

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default scenarios.map((scenario) => ({
	name: scenario.name,
	description: `should deliver concrete ${scenario.name} events while aggregating only dependencies`,
	async run() {
		const root = fs.mkdtempSync(
			path.join(fs.realpathSync(os.tmpdir()), "rspack-concrete-"),
		);
		const source = path.join(root, "src");
		fs.mkdirSync(source);
		const existing = path.join(source, "existing.js");
		const children = Array.from({ length: scenario.count }, (_, i) =>
			path.join(source, `${i}.js`),
		);
		const past = new Date(Date.now() - 10_000);
		for (const file of [
			existing,
			...(scenario.kind === "remove" || scenario.registered ? children : []),
		]) {
			fs.writeFileSync(file, "0");
			fs.utimesSync(file, past, past);
		}
		fs.utimesSync(source, past, past);
		const ignored = path.join(source, "ignored");
		const watcher = new NativeWatcher({ aggregateTimeout: 50, ignored });
		const events = [];
		const batches = [];
		watcher.watch(
			[[existing, ...(scenario.registered ? children : [])], []],
			[scenario.registered ? [] : [source], []],
			[[], []],
			BigInt(Date.now()),
			(error, batch) => {
				expect(error).toBe(null);
				batches.push(batch);
			},
			(event) => events.push(event),
		);
		try {
			await waitFor(() =>
				watcher
					.collectTimeInfoEntries()
					.fileTimestamps.entries.some((row) => row.path === existing),
			);
			await sleep(100);
			events.length = batches.length = 0;
			if (scenario.filtered) {
				fs.mkdirSync(ignored);
				for (const file of [
					path.join(ignored, "ignored.js"),
					path.join(root, "unrelated.js"),
				]) {
					fs.writeFileSync(file, "1");
					watcher.triggerEvent("create", file);
				}
			}
			for (const file of children) {
				if (scenario.kind === "remove") fs.unlinkSync(file);
				else if (!scenario.absent) fs.writeFileSync(file, "1");
				if (!scenario.native) watcher.triggerEvent(scenario.kind, file);
			}
			if (scenario.edit) {
				fs.writeFileSync(existing, "1");
				if (!scenario.native) watcher.triggerEvent("change", existing);
			}
			// Keep JS from draining callbacks while Rust processes the entire burst.
			Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 150);
			const expectedChanges = scenario.registered
				? children
				: [source, ...(scenario.edit ? [existing] : [])];
			await waitFor(() =>
				expectedChanges.every((file) =>
					batches.some((batch) => batch.changedFiles.includes(file)),
				),
			);
			const changes = new Set(batches.flatMap((batch) => batch.changedFiles));
			const removals = new Set(batches.flatMap((batch) => batch.removedFiles));
			expect([...changes].sort()).toEqual([...expectedChanges].sort());
			expect([...removals]).toEqual([]);
			const concrete = new Set(
				events
					.filter((event) => event.kind === "change")
					.map((event) => event.path),
			);
			for (const file of children) expect(concrete.has(file)).toBe(true);
			if (scenario.edit)
				expect(
					events.some(
						(event) => event.path === existing && event.kind === "change",
					),
				).toBe(true);
			if (scenario.filtered)
				expect([...new Set(events.map((event) => event.path))]).toEqual([
					existing,
				]);
		} finally {
			await watcher.close();
			fs.rmSync(root, { recursive: true, force: true });
		}
	},
}));
