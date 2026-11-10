import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const coreDir = path.dirname(require.resolve("@rspack/core/package.json"));
const Watchpack = require(path.join(coreDir, "compiled/watchpack"));
const { NativeWatcher } = require(
	require.resolve("@rspack/binding", { paths: [coreDir] })
);
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
async function until(predicate) {
	const end = Date.now() + 5000;
	while (!predicate()) {
		if (Date.now() > end) throw new Error("condition timed out");
		await sleep(10);
	}
}
async function run(operation) {
	const dir = fs.mkdtempSync(
		path.join(fs.realpathSync(os.tmpdir()), "rename-parity-")
	);
	const filename = "index.js";
	const backupName = `${filename}.map`;
	const file = path.join(dir, filename);
	const backup = path.join(dir, backupName);
	const replacement = path.join(dir, "replacement.js");
	fs.writeFileSync(file, "old");
	fs.writeFileSync(replacement, "new");
	const mtime = new Date(
		Date.now() + (operation.endsWith("future") ? 60000 : -60000)
	);
	fs.utimesSync(file, mtime, mtime);
	if (operation.includes("same")) fs.utimesSync(replacement, mtime, mtime);
	const files = [file];
	const missing = [];
	if (operation === "rename") missing.push(backup);
	if (operation === "directory") {
		fs.mkdirSync(path.join(dir, "staged/nested"), { recursive: true });
		fs.writeFileSync(path.join(dir, "staged/nested/child.js"), "child");
		missing.push(path.join(dir, "modules/nested/child.js"));
	}
	// Both backends must check the path after the synchronous rename sequence.
	const ignored = () => false;
	const wp = new Watchpack({ aggregateTimeout: 500, ignored });
	const native = new NativeWatcher({ aggregateTimeout: 500, ignored });
	const data = {
		watchpack: { raw: [], batches: [] },
		native: { raw: [], batches: [] }
	};
	const normalize = values =>
		[...values].map(p => path.relative(dir, p)).sort();
	wp.on("change", p =>
		data.watchpack.raw.push(["change", path.relative(dir, p)])
	);
	wp.on("remove", p =>
		data.watchpack.raw.push(["remove", path.relative(dir, p)])
	);
	wp.on("aggregated", (changed, removed) =>
		data.watchpack.batches.push({
			changed: normalize(changed),
			removed: normalize(removed)
		})
	);
	// Start at the epoch so both initial scans deliver a batch, even for past mtimes.
	const startTime = 0;
	native.watch(
		[files, []],
		[[], []],
		[missing, []],
		BigInt(startTime),
		(err, batch) => {
			if (err) throw err;
			data.native.batches.push({
				changed: normalize(batch.changedFiles),
				removed: normalize(batch.removedFiles)
			});
		},
		event => data.native.raw.push([event.kind, path.relative(dir, event.path)])
	);
	wp.watch({ files, missing, startTime });
	try {
		await until(() =>
			Object.values(data).every(({ batches }) => batches.length > 0)
		);
		for (const value of Object.values(data)) {
			value.raw.length = 0;
			value.batches.length = 0;
		}
		if (operation === "directory") {
			fs.renameSync(path.join(dir, "staged"), path.join(dir, "modules"));
		} else {
			if (operation.startsWith("recreate")) fs.unlinkSync(file);
			else fs.renameSync(file, backup);
			if (operation.includes("observed"))
				await until(
					() =>
						data.watchpack.raw.some(e => e[0] === "remove") &&
						data.native.raw.some(e => e[0] === "remove")
				);
			if (operation.startsWith("replace") || operation.startsWith("recreate"))
				fs.renameSync(replacement, file);
			if (operation.startsWith("roundtrip")) fs.renameSync(backup, file);
		}
		await sleep(1500);
		const expected =
			operation === "rename"
				? [{ changed: [backupName], removed: [filename] }]
				: operation === "directory"
					? [
							{
								changed: [path.join("modules", "nested", "child.js")],
								removed: []
							}
						]
					: operation === "replace-same-immediate" ||
						  operation === "roundtrip-immediate"
						? []
						: [{ changed: [filename], removed: [] }];
		expect(data.watchpack.batches).toEqual(expected);
		expect(data.native.batches).toEqual(expected);
	} finally {
		wp.close();
		await native.close();
		fs.rmSync(dir, { recursive: true, force: true });
	}
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [
	"rename",
	"replace-new-immediate",
	"replace-same-immediate",
	"replace-new-observed",
	"replace-same-observed",
	"roundtrip-immediate",
	"roundtrip-observed",
	"replace-same-immediate-future",
	"roundtrip-immediate-future",
	"recreate-same-immediate-future",
	"directory"
].map(operation => ({
	name: operation,
	description: `should aggregate ${operation} like watchpack`,
	run: () => run(operation)
}));
