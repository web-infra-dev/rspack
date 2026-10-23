import { execFileSync, spawnSync } from "node:child_process";
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
		afterCreate
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
				compiler.hooks.thisCompilation.tap("SeedBeforeModules", () =>
					beforeModules?.()
				);
			}
		]
	});
	try {
		afterCreate?.();
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
		return { seeded: seeded.output, cold: cold.output };
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
		const interrupted = staging(deadPid());
		fs.mkdirSync(interrupted);
		fs.writeFileSync(path.join(interrupted, "pack"), "interrupted");
		fs.mkdirSync(staging(process.pid));
		const result = await build(target, {
			portableCache,
			afterCreate() {
				expect(fs.existsSync(path.join(cacheLocation(target), unit))).toBe(
					false
				);
			},
			beforeModules() {
				expect(units(target)).toEqual(units(main));
			}
		});
		expectSeededFrom(result.logs, cacheLocation(main));
		expect(result.built).toEqual(["index.js"]);
		expect(
			fs
				.readdirSync(cacheLocation(target))
				.filter((name) => name.includes(".seeding-"))
		).toEqual([path.basename(staging(process.pid))]);
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
		return { seeded: result.output, cold: cold.output };
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
			"honors readonly before automatically seeding an independent compiler unit",
		skip: !gitAvailable,
		run: checkGitSeeding
	},
	{
		name: "git-worktree-output-equals-cold",
		description: "seeded output must equal a cold build after relocation",
		skip: !gitAvailable,
		async run() {
			const { seeded, cold } = await checkGitSeeding();
			expect(seeded).toEqual(cold);
		}
	},
	{
		name: "cached-modules-hit-after-seeding",
		description:
			"fresh git worktree reuses plain modules and rebuilds root-sensitive modules",
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
			"honors readonly and seed false with an explicit donor without git",
		run: checkExplicitSeeding
	},
	{
		name: "explicit-seed-output-equals-cold",
		description: "seeded output must equal a cold build after relocation",
		async run() {
			const { seeded, cold } = await checkExplicitSeeding();
			expect(seeded).toEqual(cold);
		}
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
