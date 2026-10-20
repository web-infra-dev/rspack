import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";

const gitAvailable = spawnSync("git", ["--version"]).status === 0;
const cacheLocation = (root) =>
	path.join(root, "node_modules/.cache/rspack/development");
const units = (root) =>
	fs.readdirSync(cacheLocation(root)).filter((name) => /^rspack_v_/.test(name));
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
	fs.writeFileSync(path.join(root, ".gitignore"), "node_modules\ndist\n");
	fs.writeFileSync(
		path.join(root, "index.js"),
		'import value from "./value.js"; console.log(value, import.meta.url);'
	);
	fs.writeFileSync(path.join(root, "value.js"), 'export default "seed";');
}

async function build(
	root,
	{ cache = { type: "persistent" }, portableCache = true, beforeModules } = {}
) {
	const logs = [];
	const compiler = rspack({
		context: root,
		mode: "development",
		devtool: "source-map",
		entry: "./index.js",
		incremental: false,
		output: { path: path.join(root, "dist"), clean: true },
		cache,
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
		const stats = await new Promise((resolve, reject) =>
			compiler.run((error, stats) => (error ? reject(error) : resolve(stats)))
		);
		expect(stats.hasErrors()).toBe(false);
		const logging = stats.toJson({ all: false, logging: "info" }).logging;
		for (const entry of Object.values(logging || {})) {
			for (const line of entry.entries || []) logs.push(line.message);
		}
		return { logs, output: contents(path.join(root, "dist")) };
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
		git("worktree", "add", wt, "-b", "wt");
		await build(main);
		const donor = contents(cacheLocation(main));
		let checked = false;
		const seeded = await build(wt, {
			beforeModules() {
				expect(units(wt)).toEqual(units(main));
				checked = true;
			}
		});
		expect(checked).toBe(true);
		expect(seeded.logs.join("\n")).toContain(
			`portable cache seed from ${cacheLocation(main)}`
		);
		expect(seeded.logs.join("\n")).toMatch(
			/: (directory reflink|per-file reflink or copy)/
		);
		const cold = await build(wt, { cache: false });
		expect(contents(cacheLocation(main))).toEqual(donor);
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
		const result = await build(target, {
			portableCache,
			beforeModules() {
				expect(units(target)).toEqual(units(main));
			}
		});
		expect(result.logs.join("\n")).toContain(
			`portable cache seed from ${cacheLocation(main)}`
		);
		expect(result.logs.join("\n")).toMatch(
			/: (directory reflink|per-file reflink or copy)/
		);
		const cold = await build(target, { cache: false });
		expect(contents(cacheLocation(main))).toEqual(donor);
		fs.rmSync(cacheLocation(target), { recursive: true });
		const disabled = await build(target, {
			portableCache: { ...portableCache, seed: false }
		});
		expect(disabled.logs.join("\n")).not.toContain("portable cache seed from");
		return { seeded: result.output, cold: cold.output };
	});
}

export default [
	{
		name: "option-defaults",
		description:
			"portable cache sets hash fallback defaults and rejects portable false",
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
					experiments: { portableCache: true },
					snapshot: { resolve: { hash: true } }
				});
				try {
					expect(compiler.options.cache.portable).toEqual({ root });
					expect(compiler.options.snapshot.module).toEqual({
						timestamp: true,
						hash: true
					});
					expect(compiler.options.snapshot.contextModule).toEqual({
						timestamp: true,
						hash: true
					});
					expect(compiler.options.snapshot.resolve).toEqual({ hash: true });
				} finally {
					await new Promise((resolve) => compiler.close(resolve));
				}
			});
		}
	},
	{
		name: "git-worktree-mechanics",
		description:
			"seeds a separate compiler unit before modules build without modifying the donor",
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
		name: "cached-modules-hit-after-seeding-requires-pr4",
		description:
			"cached modules hit after seeding: skipped until PR4 root binding and identity sweep are restacked",
		skip: true,
		run() {}
	},
	{
		name: "outside-root-warning",
		description:
			"warns and skips seeding for storage outside the portable root",
		async run() {
			await withFixture(async (temp) => {
				const root = path.join(temp, "root");
				fixture(root);
				const result = await build(root, {
					cache: {
						type: "persistent",
						storage: { directory: path.join(temp, "shared") }
					}
				});
				expect(result.logs.join("\n")).toContain(
					"must be inside the portable root"
				);
				expect(result.logs.join("\n")).not.toContain(
					"portable cache seed from"
				);
			});
		}
	},
	{
		name: "explicit-seed-without-git",
		description:
			"seeds from an explicit location without git and honors seed false",
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
	{
		name: "pending-donor-transaction",
		description:
			"falls back to a cold build without recovering or changing a busy donor",
		async run() {
			await withFixture(async (temp) => {
				const main = path.join(temp, "main");
				const target = path.join(temp, "target");
				fixture(main);
				fixture(target);
				await build(main);
				const transaction = path.join(
					cacheLocation(main),
					units(main)[0],
					".temp"
				);
				fs.mkdirSync(transaction, { recursive: true });
				const pid = Buffer.alloc(4);
				pid.writeUInt32BE(process.pid);
				fs.writeFileSync(
					path.join(transaction, "state.lock"),
					Buffer.concat([pid, Buffer.from("node")])
				);
				const donor = contents(cacheLocation(main));
				const result = await build(target, {
					portableCache: { seedFrom: cacheLocation(main) }
				});
				expect(result.logs.join("\n")).toContain(
					"skipped: donor has a pending transaction"
				);
				expect(result.output).toEqual(
					(await build(target, { cache: false })).output
				);
				expect(contents(cacheLocation(main))).toEqual(donor);
			});
		}
	}
];
