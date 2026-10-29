import { createRequire } from "node:module";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = require("@rspack/binding");

it("delivers all raw events before their aggregate while JS is blocked", async () => {
	const dir = fs.mkdtempSync(
		path.join(fs.realpathSync(os.tmpdir()), "rspack-delivery-"),
	);
	const files = Array.from({ length: 32 }, (_, i) => path.join(dir, `${i}.js`));
	for (const file of files) fs.writeFileSync(file, "0");
	const watcher = new NativeWatcher({ aggregateTimeout: 20 });
	const events = [];
	const watch = (generation) =>
		watcher.watch(
			[files, []],
			[[], []],
			[[], []],
			BigInt(Date.now()),
			(error, batch) => {
				expect(error).toBe(null);
				events.push({ batch, generation });
			},
			(event) => events.push({ raw: event.path, generation }),
		);
	watch(1);
	try {
		await sleep(200);
		events.length = 0;
		for (const file of files) {
			fs.writeFileSync(file, "1");
			watcher.triggerEvent("change", file);
		}
		// Rust continues queuing deliveries while the JS thread cannot drain them.
		Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 200);
		// Replace callbacks before JS drains the first generation's queued messages.
		watch(2);
		await sleep(200);
		const raw = new Set();
		const aggregated = new Set();
		for (const event of events) {
			expect(event.generation).toBe(1);
			if (event.raw) raw.add(event.raw);
			else
				for (const file of event.batch.changedFiles) {
					expect(raw.has(file)).toBe(true);
					aggregated.add(file);
				}
		}
		expect([...raw].sort()).toEqual([...files].sort());
		expect([...aggregated].sort()).toEqual([...files].sort());
		events.length = 0;
		fs.writeFileSync(files[0], "2");
		watcher.triggerEvent("change", files[0]);
		await sleep(200);
		expect(events.some((event) => event.raw === files[0])).toBe(true);
		expect(
			events.some((event) => event.batch?.changedFiles.includes(files[0])),
		).toBe(true);
		expect(events.every((event) => event.generation === 2)).toBe(true);
	} finally {
		await watcher.close();
		fs.rmSync(dir, { recursive: true, force: true });
	}
});
