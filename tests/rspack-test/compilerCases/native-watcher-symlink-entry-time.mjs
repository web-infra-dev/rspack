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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
	["replace", "startup", "old-link", "dangling", "missing"].map((scenario) => ({
		name: `${native ? "native" : "watchpack"}-${scenario}`,
		description:
			"should observe file symlinks themselves when following is disabled",
		async run() {
			const root = fs.mkdtempSync(
				path.join(fs.realpathSync(os.tmpdir()), "rspack-link-entry-"),
			);
			const app = path.join(root, "app");
			const outside = path.join(root, "outside");
			fs.mkdirSync(app);
			fs.mkdirSync(outside);
			const past = new Date("2021-01-01T00:00:00Z");
			const targets = ["old.js", "new.js"].map((name) => {
				const file = path.join(outside, name);
				fs.writeFileSync(file, name);
				fs.utimesSync(file, past, past);
				return file;
			});
			const startTime = Date.now();
			await sleep(25);
			const link = path.join(app, "link.js");
			fs.symlinkSync(targets[0], link, "file");
			if (scenario === "replace" || scenario === "old-link")
				fs.lutimesSync(link, past, past);
			if (scenario === "old-link")
				fs.writeFileSync(targets[0], "new target contents");
			if (scenario === "dangling") fs.unlinkSync(targets[0]);
			const options = { followSymlinks: false, aggregateTimeout: 40 };
			const watcher = native
				? new NativeWatcher(options)
				: new Watchpack(options);
			const changes = new Set();
			const removals = new Set();
			let failure;
			const record = (changed, removed) => {
				for (const file of changed) changes.add(file);
				for (const file of removed) removals.add(file);
			};
			const until = async (predicate) => {
				const deadline = Date.now() + 5000;
				while (!predicate() && !failure && Date.now() < deadline)
					await sleep(10);
				expect(failure).toBeUndefined();
				expect(predicate()).toBe(true);
			};
			const timestamp = () => {
				if (native)
					return watcher
						.collectTimeInfoEntries()
						.fileTimestamps.entries.find((entry) => entry.path === link)
						?.timestamp;
				const files = new Map();
				watcher.collectTimeInfoEntries(files, new Map());
				return files.get(link)?.timestamp;
			};
			try {
				const files = scenario === "missing" ? [] : [link];
				const missing = scenario === "missing" ? [link] : [];
				if (native)
					await watcher.watch(
						[files, []],
						[[], []],
						[missing, []],
						BigInt(startTime),
						(error, batch) => {
							if (error) failure = error;
							else record(batch.changedFiles, batch.removedFiles);
						},
						() => {},
					);
				else {
					watcher.on("aggregated", record);
					watcher.watch({ files, missing, startTime });
				}
				if (scenario === "replace" || scenario === "old-link") {
					await sleep(500);
					expect([...changes]).toEqual([]);
				} else await until(() => changes.has(link));
				expect(timestamp()).toBe(fs.lstatSync(link).mtime.getTime());
				if (scenario === "replace") {
					if (process.platform === "linux") {
						fs.lchownSync(link, process.getuid(), process.getgid());
						await sleep(300);
						expect([...changes]).toEqual([]);
					}
					const replacement = path.join(app, "replacement.js");
					fs.symlinkSync(targets[1], replacement, "file");
					fs.renameSync(replacement, link);
					await until(() => changes.has(link));
					expect(fs.readFileSync(link, "utf8")).toBe("new.js");
					expect(timestamp()).toBe(fs.lstatSync(link).mtime.getTime());
				}
				expect([...removals]).toEqual([]);
				expect(failure).toBeUndefined();
			} finally {
				await watcher.close();
				fs.rmSync(root, { recursive: true, force: true });
			}
		},
	})),
);
