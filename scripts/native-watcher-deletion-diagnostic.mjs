import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { promisify } from "node:util";

const require = createRequire(import.meta.url);
const diagnosticDir = process.env.DIAGNOSTIC_DIR;
const bindingDir = path.join(diagnosticDir, "binding");
const binding = require(
	path.join(
		bindingDir,
		fs.readdirSync(bindingDir).find((file) => file.endsWith(".node")),
	),
);
const Watchpack = require(path.join(diagnosticDir, "node_modules/watchpack"));
const execFileAsync = promisify(execFile);
const modes = ["sync", "split", "exec"];
const scenarios = ["context", "file", "missing"];
const fixtures = [];

binding.registerGlobalTrace(
	"rspack_watcher::fs_event=debug",
	"logger",
	path.join(diagnosticDir, "notify-events.log"),
);

async function waitFor(predicate, label) {
	const deadline = Date.now() + 5000;
	while (!predicate() && Date.now() < deadline) await sleep(5);
	assert.ok(predicate(), label);
}

for (const mode of modes) {
	for (const scenario of scenarios) {
		for (let round = 0; round < Number(process.env.ROUNDS ?? 10); round++) {
			const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-delete-"));
			const source = path.join(root, "src");
			const child = path.join(source, "child");
			const inside = path.join(child, "inside.js");
			const ready = path.join(source, "ready.js");
			const scanned = path.join(source, "scanned.js");
			const ignored = path.join(source, "ignored.js");
			fs.mkdirSync(child, { recursive: true });
			for (const file of [inside, ready, scanned, ignored])
				fs.writeFileSync(file, "0");
			const past = new Date(Date.now() - 10_000);
			for (const file of [inside, ready, scanned, ignored, child, source])
				fs.utimesSync(file, past, past);
			fixtures.push({
				mode,
				scenario,
				round,
				root,
				source,
				child,
				inside,
				ready,
				scanned,
				ignored,
			});
		}
	}
}
await sleep(2100);

const results = [];
for (const fixture of fixtures) {
	const {
		mode,
		scenario,
		root,
		source,
		child,
		inside,
		ready,
		scanned,
		ignored,
	} = fixture;
	const files = [ready, ignored, ...(scenario === "file" ? [inside] : [])];
	const directories = [source, child];
	const missing = scenario === "missing" ? [inside] : [];
	const options = {
		aggregateTimeout: 50,
		ignored: ignored.replaceAll("\\", "/"),
	};
	const native = new binding.NativeWatcher(options);
	const watchpack = new Watchpack(options);
	const events = [],
		batches = [],
		operations = [];
	let phase = "startup";
	const record = (watcher, kind, file, mtime, type) =>
		events.push({
			watcher,
			kind,
			file: path.relative(source, file),
			mtime,
			type,
			time: Date.now(),
			phase,
			childExists: fs.existsSync(child),
			insideExists: fs.existsSync(inside),
		});
	native.watch(
		[files, []],
		[directories, []],
		[missing, []],
		BigInt(Date.now()),
		(error, result) => {
			assert.equal(error, null);
			batches.push({
				watcher: "native",
				phase,
				time: Date.now(),
				changes: result.changedFiles.map((p) => path.relative(source, p)),
				removals: result.removedFiles.map((p) => path.relative(source, p)),
			});
		},
		(event) =>
			record(
				"native",
				event.kind,
				event.path,
				event.kind === "change"
					? event.mtime === null
						? null
						: "number"
					: undefined,
			),
	);
	watchpack.on("change", (file, mtime, type) =>
		record("watchpack", "change", file, mtime === null ? null : "number", type),
	);
	watchpack.on("remove", (file, type) =>
		record("watchpack", "remove", file, undefined, type),
	);
	watchpack.on("aggregated", (changes, removals) =>
		batches.push({
			watcher: "watchpack",
			phase,
			time: Date.now(),
			changes: [...changes].map((p) => path.relative(source, p)),
			removals: [...removals].map((p) => path.relative(source, p)),
		}),
	);
	watchpack.watch({ files, directories, missing, startTime: Date.now() });
	try {
		await waitFor(
			() =>
				native
					.collectTimeInfoEntries()
					.fileTimestamps.entries.some((entry) => entry.path === scanned) &&
				[...watchpack.directoryWatchers.values()].every(
					({ watcher }) => !watcher.directoryWatcher.initialScan,
				),
			"initial scans",
		);
		events.length = batches.length = 0;
		phase = mode;
		operations.push({ operation: mode, edge: "start", time: Date.now() });
		if (mode === "sync") {
			fs.rmSync(child, { recursive: true });
		} else if (mode === "exec") {
			const rm =
				process.platform === "win32"
					? path.join(process.env.ProgramFiles, "Git/usr/bin/rm.exe")
					: "/bin/rm";
			await execFileAsync(rm, ["-rf", child]);
		} else {
			fs.unlinkSync(inside);
			operations.push({ operation: "unlink", edge: "end", time: Date.now() });
			await waitFor(
				() =>
					batches.some((batch) => batch.watcher === "native") &&
					batches.some((batch) => batch.watcher === "watchpack"),
				"unlink batches",
			);
			await sleep(100);
			assert.ok(fs.existsSync(child));
			phase = "rmdir";
			operations.push({ operation: "rmdir", edge: "start", time: Date.now() });
			fs.rmdirSync(child);
		}
		operations.push({ operation: phase, edge: "end", time: Date.now() });
		fs.unlinkSync(ignored);
		await waitFor(
			() =>
				batches.some(
					(batch) => batch.phase === phase && batch.watcher === "native",
				) &&
				batches.some(
					(batch) => batch.phase === phase && batch.watcher === "watchpack",
				),
			"removal batches",
		);
		await sleep(100);
		const result = { ...fixture, events, batches, operations };
		results.push(result);
		const line = JSON.stringify(result);
		fs.appendFileSync(path.join(diagnosticDir, "events.ndjson"), `${line}\n`);
		console.log(line);
	} finally {
		watchpack.close();
		await native.close();
		fs.rmSync(root, { recursive: true, force: true });
	}
}
binding.cleanupGlobalTrace();
console.log(
	JSON.stringify({
		summary: true,
		cases: results.length,
		numericChild: results.filter((result) =>
			result.events.some(
				(event) =>
					event.file === "child" &&
					event.kind === "change" &&
					event.mtime === "number",
			),
		).length,
	}),
);
