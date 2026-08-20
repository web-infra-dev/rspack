const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const coreDir = path.dirname(require.resolve("@rspack/core/package.json"));
const bindingPath = require.resolve("@rspack/binding", { paths: [coreDir] });
const binding = require(bindingPath);

const AGGREGATE_TIMEOUT = 200;
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));

// `os.tmpdir()` is a symlink on macOS; the watcher reports real paths.
const makeDir = () =>
	fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "rspack-ignored-fn-"));

describe("NativeWatcher `ignored` as a function", () => {
	it("is asked from the watcher's own threads and drops the entries it rejects", async () => {
		const dir = makeDir();
		const kept = path.join(dir, "kept.js");
		const rejected = path.join(dir, "rejected.js");
		fs.writeFileSync(kept, "0");
		fs.writeFileSync(rejected, "0");
		await sleep(200);

		const asked = [];
		const watcher = new binding.NativeWatcher({
			aggregateTimeout: AGGREGATE_TIMEOUT,
			ignored: entry => {
				asked.push(entry);
				return entry.endsWith("rejected.js");
			}
		});

		const batches = [];
		watcher.watch(
			[[kept, rejected], []],
			[[], []],
			[[], []],
			BigInt(Date.now()),
			(err, result) => {
				if (err) throw err;
				batches.push(result);
			},
			() => {}
		);

		await sleep(500);
		fs.writeFileSync(rejected, "1");
		fs.writeFileSync(kept, "1");
		await sleep(1500);

		try {
			// Rust reached the JS predicate — registration runs on the watcher
			// thread, so this only works through the blocking dispatch.
			expect(asked).toContain(kept);
			expect(asked).toContain(rejected);

			const changed = batches.flatMap(batch => batch.changedFiles);
			expect(changed).toContain(kept);
			expect(changed).not.toContain(rejected);
		} finally {
			await watcher.close();
			fs.rmSync(dir, { recursive: true, force: true });
		}
	});

	// Run in a child process: without the JS-thread guard `triggerEvent` waits on
	// the event loop while occupying it, which wedges the whole thread — the
	// in-process timeout could not fire either. The child turns that into a
	// bounded, reportable failure.
	it("does not deadlock when `triggerEvent` re-enters from the JS thread", () => {
		const dir = makeDir();
		const file = path.join(dir, "injected.js");
		fs.writeFileSync(file, "0");

		const script = `
			const binding = require(process.argv[1]);
			const asked = [];
			const watcher = new binding.NativeWatcher({
				aggregateTimeout: ${AGGREGATE_TIMEOUT},
				ignored: entry => { asked.push(entry); return true; }
			});
			watcher.triggerEvent("change", process.argv[2]);
			console.log(JSON.stringify({ returned: true, asked }));
			watcher.close().then(() => process.exit(0));
		`;

		try {
			const child = spawnSync(process.execPath, [
				"-e",
				script,
				bindingPath,
				file
			], { timeout: 15000, encoding: "utf8" });

			expect(child.signal).toBe(null);
			const report = JSON.parse(child.stdout.trim());
			expect(report.returned).toBe(true);
			// Injected events are explicit plugin requests, so the predicate is
			// not consulted for them.
			expect(report.asked).not.toContain(file);
		} finally {
			fs.rmSync(dir, { recursive: true, force: true });
		}
	});
});
