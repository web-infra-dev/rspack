import { execFileSync, spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";

const gitAvailable = spawnSync("git", ["--version"]).status === 0;
const cacheLocation = (root) =>
	path.join(root, "node_modules/.cache/rspack/development");
const units = (root) =>
	fs
		.readdirSync(cacheLocation(root))
		.filter((name) => /^rspack_v_[0-9a-f]{16}$/.test(name));
const seedLogs = (logs) =>
	logs.filter((entry) => /portable cache (seeded from|not seeded)/.test(entry));
const expectSeededFrom = (logs, donor) => {
	const match = logs
		.map((entry) => entry.match(/portable cache seeded from (.*)$/))
		.find(Boolean);
	expect(match).toBeTruthy();
	expect(fs.realpathSync.native(match[1])).toBe(fs.realpathSync.native(donor));
};
const deadPid = () => spawnSync(process.execPath, ["-e", ""]).pid;
const DAY = 24 * 60 * 60 * 1000;
const stagingDirs = (root) =>
	fs
		.readdirSync(cacheLocation(root))
		.filter((name) => name.includes(".seeding-"))
		.sort();
async function withLiveProcess(run) {
	const child = spawn(process.execPath, ["-e", "setTimeout(() => {}, 120000)"], {
		stdio: "ignore"
	});
	await new Promise((resolve, reject) => {
		child.once("spawn", resolve);
		child.once("error", reject);
	});
	try {
		return await run(child.pid);
	} finally {
		child.kill();
	}
}
function plantStaging(root, unit, owner, tag, ageMs = 0) {
	const dir = path.join(cacheLocation(root), `${unit}.seeding-${owner}-${tag}`);
	fs.mkdirSync(dir, { recursive: true });
	fs.writeFileSync(path.join(dir, "pack"), tag);
	const time = new Date(Date.now() - ageMs);
	fs.utimesSync(dir, time, time);
	return path.basename(dir);
}
const unreadablePacks =
	process.platform !== "win32" && process.getuid?.() !== 0;
function makePackUnreadable(unit) {
	const pack = Object.keys(contents(unit)).find(
		(file) =>
			!file.startsWith(`meta${path.sep}`) &&
			path.basename(file) !== "_meta" &&
			!file.startsWith(".")
	);
	expect(pack).toBeDefined();
	fs.chmodSync(path.join(unit, pack), 0);
	return () => fs.chmodSync(path.join(unit, pack), 0o644);
}
const processName = () =>
	process.platform === "win32"
		? path.basename(process.execPath)
		: execFileSync("ps", ["-p", String(process.pid), "-o", "comm="])
				.toString()
				.trim();
function writeStateLock(unit, pid, name) {
	const transaction = path.join(unit, ".temp");
	fs.mkdirSync(transaction, { recursive: true });
	const bytes = Buffer.alloc(4);
	bytes.writeUInt32BE(pid);
	fs.writeFileSync(
		path.join(transaction, "state.lock"),
		Buffer.concat([bytes, Buffer.from(name)])
	);
}
const contents = (root) =>
	Object.fromEntries(
		fs
			.readdirSync(root, { recursive: true, withFileTypes: true })
			.filter((entry) => entry.isFile())
			.map((entry) => {
				const file = path.join(entry.parentPath, entry.name);
				return [
					path.relative(root, file),
					fs.readFileSync(file).toString("base64")
				];
			})
			.sort(([a], [b]) => a.localeCompare(b))
	);

function fixture(root) {
	fs.mkdirSync(root, { recursive: true });
	fs.writeFileSync(
		path.join(root, "package.json"),
		'{"name":"portable-cache-seed"}'
	);
	fs.writeFileSync(
		path.join(root, ".gitignore"),
		"node_modules\ndist\n.builds\n"
	);
	fs.writeFileSync(
		path.join(root, "index.js"),
		'import value from "./value.js"; import context from "./context.js"; import asset from "./asset.txt"; console.log(value, import.meta.url, context, asset);'
	);
	fs.writeFileSync(
		path.join(root, "value.js"),
		'import value from "./value-two.js"; export default value;'
	);
	fs.writeFileSync(path.join(root, "value-two.js"), 'export default "seed";');
	fs.writeFileSync(
		path.join(root, "context.js"),
		'const context = require.context("./children", false, /\\.js$/); export default context.keys().map(context);'
	);
	fs.mkdirSync(path.join(root, "children"));
	fs.writeFileSync(
		path.join(root, "children/child.js"),
		'module.exports = "child";'
	);
	fs.writeFileSync(path.join(root, "asset.txt"), "portable asset");
	fs.writeFileSync(
		path.join(root, "count.cjs"),
		`const fs = require("node:fs");
const path = require("node:path");
module.exports = function(source) {
  fs.appendFileSync(path.join(this.rootContext, ".builds"), path.relative(this.rootContext, this.resourcePath).split(path.sep).join("/") + "\\n");
  return source;
};`
	);
}

async function build(
	root,
	{
		cache = { type: "persistent" },
		portableCache = true,
		beforeModules,
		beforeRun
	} = {}
) {
	const logs = [];
	fs.writeFileSync(path.join(root, ".builds"), "");
	const compiler = rspack({
		context: root,
		mode: "development",
		devtool: "source-map",
		entry: "./index.js",
		incremental: false,
		output: { path: path.join(root, "dist"), clean: true },
		cache,
		module: {
			rules: [
				{ test: /\.js$/, use: path.join(root, "count.cjs") },
				{ test: /\.txt$/, type: "asset/resource" }
			]
		},
		experiments: { portableCache },
		infrastructureLogging: {
			level: "info",
			console: {
				...console,
				info: (...args) => logs.push(args.join(" ")),
				warn: (...args) => logs.push(args.join(" ")),
				log: (...args) => logs.push(args.join(" "))
			}
		},
		plugins: [
			(compiler) => {
				compiler.hooks.beforeRun.tap("SeedBeforeRun", () => beforeRun?.());
				compiler.hooks.thisCompilation.tap("SeedBeforeModules", () =>
					beforeModules?.()
				);
			}
		]
	});
	try {
		const stats = await new Promise((resolve, reject) =>
			compiler.run((error, stats) => (error ? reject(error) : resolve(stats)))
		);
		expect(stats.hasErrors()).toBe(false);
		const logging = stats.toJson({ all: false, logging: "info" }).logging;
		for (const entry of Object.values(logging || {})) {
			for (const line of entry.entries || []) logs.push(line.message);
		}
		return {
			logs,
			modules: stats.toJson({ all: false, modules: true, cachedModules: true })
				.modules,
			built: fs
				.readFileSync(path.join(root, ".builds"), "utf8")
				.trim()
				.split("\n")
				.filter(Boolean)
				.sort(),
			output: contents(path.join(root, "dist"))
		};
	} finally {
		await new Promise((resolve, reject) =>
			compiler.close((error) => (error ? reject(error) : resolve()))
		);
	}
}

async function withFixture(run) {
	const temp = fs.realpathSync(
		fs.mkdtempSync(path.join(os.tmpdir(), "rspack-seed-"))
	);
	try {
		return await run(temp);
	} finally {
		fs.rmSync(temp, { recursive: true, force: true });
	}
}

async function checkReadonlySeeding(root, donorRoot, portableCache) {
	const donor = contents(cacheLocation(donorRoot));
	expect(fs.existsSync(cacheLocation(root))).toBe(false);
	const result = await build(root, {
		cache: { type: "persistent", readonly: true },
		portableCache,
		beforeModules() {
			expect(contents(cacheLocation(root))).toEqual({});
		}
	});
	expect(result.built).toHaveLength(5);
	expect(seedLogs(result.logs)).toEqual([]);
	expect(contents(cacheLocation(root))).toEqual({});
	const warm = await build(donorRoot, {
		cache: { type: "persistent", readonly: true }
	});
	expect(warm.built).toEqual([]);
	expect(contents(cacheLocation(donorRoot))).toEqual(donor);
	fs.rmSync(cacheLocation(root), { recursive: true });
}

async function checkGitSeeding() {
	return await withFixture(async (temp) => {
		const main = path.join(temp, "main");
		const wt = path.join(temp, "deeper", "wt");
		fixture(main);
		const git = (...args) =>
			execFileSync("git", args, { cwd: main, stdio: "pipe" });
		git("init");
		git("add", ".");
		git(
			"-c",
			"user.name=Rspack",
			"-c",
			"user.email=rspack@example.com",
			"-c",
			"commit.gpgsign=false",
			"commit",
			"-m",
			"fixture"
		);
		const sources = [
			"index.js",
			"value.js",
			"value-two.js",
			"context.js",
			"children/child.js"
		];
		for (const source of [
			...sources,
			"count.cjs",
			"asset.txt",
			"package.json"
		]) {
			fs.utimesSync(
				path.join(main, source),
				new Date("2020-01-01"),
				new Date("2020-01-01")
			);
		}
		const original = await build(main);
		expect(original.built).toEqual([...sources].sort());
		git("worktree", "add", wt, "-b", "wt");
		for (const source of sources) {
			expect(fs.readFileSync(path.join(wt, source))).toEqual(
				fs.readFileSync(path.join(main, source))
			);
			expect(fs.statSync(path.join(wt, source)).mtimeMs).not.toBe(
				fs.statSync(path.join(main, source)).mtimeMs
			);
		}
		const donor = contents(cacheLocation(main));
		await checkReadonlySeeding(wt, main, true);
		let checked = false;
		const seeded = await build(wt, {
			beforeModules() {
				expect(units(wt)).toEqual(units(main));
				for (const unit of units(main)) {
					const donorUnit = path.join(cacheLocation(main), unit);
					const targetUnit = path.join(cacheLocation(wt), unit);
					expect(contents(targetUnit)).toEqual(contents(donorUnit));
					for (const relative of ["", ...Object.keys(contents(donorUnit))]) {
						const a = fs.statSync(path.join(donorUnit, relative));
						const b = fs.statSync(path.join(targetUnit, relative));
						expect([b.dev, b.ino]).not.toEqual([a.dev, a.ino]);
					}
				}
				checked = true;
			}
		});
		expect(checked).toBe(true);
		expectSeededFrom(seeded.logs, cacheLocation(main));
		const context = seeded.modules.find((module) =>
			module.name?.includes("./children|sync")
		);
		expect(context).toBeDefined();
		expect(context.built).toBe(false);
		const cold = await build(wt, { cache: false });
		expect(cold.built).toEqual([...sources].sort());
		expect(Object.keys(seeded.output)).toEqual(
			expect.arrayContaining(["main.js", "main.js.map"])
		);
		expect(
			Object.keys(seeded.output).some((name) => name.endsWith(".txt"))
		).toBe(true);
		expect(seeded.output).toEqual(cold.output);
		expect(contents(cacheLocation(main))).toEqual(donor);
		expect(seeded.built).toEqual(["index.js"]);
	});
}

async function checkExplicitSeeding() {
	return await withFixture(async (temp) => {
		const main = path.join(temp, "main");
		const target = path.join(temp, "target");
		fixture(main);
		fixture(target);
		await build(main);
		const donor = contents(cacheLocation(main));
		const portableCache = { seedFrom: cacheLocation(main) };
		await checkReadonlySeeding(target, main, portableCache);
		const [unit] = units(main);
		fs.mkdirSync(cacheLocation(target), { recursive: true });
		const staging = (owner) =>
			path.join(cacheLocation(target), `${unit}.seeding-${owner}-planted`);
		plantStaging(target, unit, deadPid(), "interrupted");
		fs.mkdirSync(staging(process.pid));
		const result = await withLiveProcess(async (owner) => {
			const live = plantStaging(target, unit, owner, "live", 2 * DAY);
			const result = await build(target, {
				portableCache,
				beforeRun() {
					expect(units(target)).toEqual([]);
				},
				beforeModules() {
					expect(units(target)).toEqual(units(main));
				}
			});
			expect(stagingDirs(target)).toEqual(
				[path.basename(staging(process.pid)), live].sort()
			);
			return result;
		});
		expectSeededFrom(result.logs, cacheLocation(main));
		expect(result.built).toEqual(["index.js"]);
		const cold = await build(target, { cache: false });
		expect(result.output).toEqual(cold.output);
		expect(contents(cacheLocation(main))).toEqual(donor);
		fs.rmSync(cacheLocation(target), { recursive: true });
		const disabled = await build(target, {
			portableCache: { ...portableCache, seed: false }
		});
		expect(seedLogs(disabled.logs)).toEqual([]);
		expect(disabled.built).toEqual(cold.built);
		expect(disabled.output).toEqual(cold.output);
	});
}

export default [
	{
		name: "option-defaults",
		description:
			"portable cache implies cache.portable without changing snapshot defaults",
		async run() {
			await withFixture(async (root) => {
				fixture(root);
				expect(() =>
					rspack({
						context: root,
						cache: { type: "persistent", portable: false },
						experiments: { portableCache: true }
					})
				).toThrow("requires cache.portable");
				const compiler = rspack({
					context: root,
					mode: "development",
					cache: { type: "persistent" },
					experiments: { portableCache: true }
				});
				const plain = rspack({
					context: root,
					mode: "development",
					cache: { type: "persistent" }
				});
				try {
					expect(fs.realpathSync.native(compiler.options.cache.portable.root)).toBe(
						fs.realpathSync.native(root)
					);
					expect(compiler.options.snapshot).toEqual(plain.options.snapshot);
				} finally {
					await new Promise((resolve) => compiler.close(resolve));
					await new Promise((resolve) => plain.close(resolve));
				}
			});
		}
	},
	{
		name: "git-worktree-mechanics",
		description:
			"seeds an independent worktree unit, honors readonly, reuses modules and matches cold output",
		skip: !gitAvailable,
		run: checkGitSeeding
	},
	{
		name: "outside-root-warning",
		description:
			"warns and skips seeding for storage outside the portable root",
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const root = path.join(temp, "root");
				fixture(main);
				fixture(root);
				await build(main);
				const result = await build(root, {
					cache: {
						type: "persistent",
						storage: { directory: path.join(temp, "shared") }
					},
					portableCache: { seedFrom: cacheLocation(main) }
				});
				expect(result.logs.join("\n")).toContain(
					"must be inside the portable root"
				);
				expect(seedLogs(result.logs)).toEqual([]);
				expect(result.built).toHaveLength(5);
			});
		}
	},
	{
		name: "explicit-seed-without-git",
		description:
			"honors readonly and seed false with an explicit donor and matches cold output",
		run: checkExplicitSeeding
	},
	...[
		{
			name: "live-donor-transaction",
			description: "skips a donor whose transaction owner is running",
			prepare: (unit) => writeStateLock(unit, process.pid, processName()),
			reason: "donor has a running transaction"
		},
		{
			name: "unfinished-donor-commit",
			description: "skips a donor whose crashed commit still needs recovery",
			prepare(unit) {
				writeStateLock(unit, deadPid(), "node");
				fs.writeFileSync(
					path.join(unit, ".temp", "commit.lock"),
					"[ADD]\n[REMOVE]\n"
				);
			},
			reason: "donor has an unfinished commit"
		},
		{
			name: "stale-donor-transaction",
			description: "seeds from a donor whose transaction owner has exited",
			prepare: (unit) => writeStateLock(unit, deadPid(), processName())
		}
	].map(({ name, description, prepare, reason }) => ({
		name,
		description,
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const target = path.join(temp, "target");
				fixture(main);
				fixture(target);
				await build(main);
				prepare(path.join(cacheLocation(main), units(main)[0]));
				const donor = contents(cacheLocation(main));
				const result = await build(target, {
					portableCache: { seedFrom: cacheLocation(main) }
				});
				if (reason) {
					expect(result.logs.join("\n")).toContain(
						`portable cache not seeded: ${fs.realpathSync.native(cacheLocation(main))} (${reason})`
					);
					expect(result.built).toHaveLength(5);
				} else {
					expectSeededFrom(result.logs, cacheLocation(main));
					expect(result.built).toEqual(["index.js"]);
				}
				expect(result.output).toEqual(
					(await build(target, { cache: false })).output
				);
				expect(contents(cacheLocation(main))).toEqual(donor);
			});
		}
	})),
	{
		name: "unknown-owner-liveness",
		description:
			"keeps recent staging and removes day-old staging when process lookup is unavailable",
		skip: process.platform === "win32",
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const target = path.join(temp, "target");
				const emptyPath = path.join(temp, "empty-path");
				fixture(main);
				fixture(target);
				fs.mkdirSync(emptyPath);
				await build(main);
				const [unit] = units(main);
				await withLiveProcess(async (owner) => {
					const recent = plantStaging(target, unit, owner, "recent");
					plantStaging(target, unit, owner, "old", 2 * DAY);
					const path_ = process.env.PATH;
					process.env.PATH = emptyPath;
					let result;
					try {
						result = await build(target, {
							portableCache: { seedFrom: cacheLocation(main) }
						});
					} finally {
						process.env.PATH = path_;
					}
					expectSeededFrom(result.logs, cacheLocation(main));
					expect(stagingDirs(target)).toEqual([recent]);
				});
			});
		}
	},
	...[
		{
			name: "unreadable-donor-pack",
			description:
				"removes the staging copy and builds cold when copying a donor fails",
			reason: /\(.*(permission denied|EACCES).*\)/i
		},
		{
			name: "version-mismatch-before-copy",
			description:
				"rejects a donor on cache version before copying its packs",
			version: "other",
			reason: /\(cache version does not match\)/
		}
	].map(({ name, description, version, reason }) => ({
		name,
		description,
		skip: !unreadablePacks,
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const target = path.join(temp, "target");
				fixture(main);
				fixture(target);
				await build(main, { cache: { type: "persistent", version } });
				const restore = makePackUnreadable(
					path.join(cacheLocation(main), units(main)[0])
				);
				try {
					const result = await build(target, {
						portableCache: { seedFrom: cacheLocation(main) }
					});
					const outcome = seedLogs(result.logs).join("\n");
					expect(outcome).toContain(
						`portable cache not seeded: ${fs.realpathSync.native(cacheLocation(main))} (`
					);
					expect(outcome).toMatch(reason);
					expect(stagingDirs(target)).toEqual([]);
					expect(result.built).toHaveLength(5);
					expect(result.output).toEqual(
						(await build(target, { cache: false })).output
					);
				} finally {
					restore();
				}
			});
		}
	})),
	{
		name: "version-mismatched-donor",
		description:
			"skips the newest worktree cache when its cache version differs",
		skip: !gitAvailable,
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const other = path.join(temp, "other");
				const fresh = path.join(temp, "deeper", "fresh");
				fixture(main);
				const git = (...args) =>
					execFileSync("git", args, { cwd: main, stdio: "pipe" });
				git("init");
				git("add", ".");
				git(
					"-c",
					"user.name=Rspack",
					"-c",
					"user.email=rspack@example.com",
					"-c",
					"commit.gpgsign=false",
					"commit",
					"-m",
					"fixture"
				);
				await build(main);
				git("worktree", "add", other, "-b", "other");
				await build(other, {
					cache: { type: "persistent", version: "other" },
					portableCache: { seed: false }
				});
				git("worktree", "add", fresh, "-b", "fresh");
				const result = await build(fresh);
				expectSeededFrom(result.logs, cacheLocation(main));
				expect(result.built).toEqual(["index.js"]);
			});
		}
	}
];
