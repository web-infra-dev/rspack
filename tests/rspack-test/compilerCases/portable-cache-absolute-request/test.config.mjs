import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

const LOADER_RUNS = Symbol.for("portable-cache-absolute-request-loader-runs");

// An unmodified third-party loader: it writes a path it resolved for this checkout and declares
// nothing. The project root is the parent of `context`, so the path is outside `context`.
const LOADER = String.raw`
const path = require("node:path");
module.exports = function (source) {
	globalThis[Symbol.for("portable-cache-absolute-request-loader-runs")].push(this.resourcePath);
	const api = JSON.stringify(path.join(this.rootContext, "../node_modules/api/index.js"));
	switch (path.basename(this.resourcePath)) {
		case "required.gen":
			return "var api = require(" + api + "); module.exports = api;";
		case "reexport.gen":
			return "module.exports = require(" + api + ");";
		case "keyed.gen":
			return "module.exports = { [" + api + "]: 1 }; module.exports.value = require(" + api + ");";
		case "listed.gen":
			// The path comes from a file, so it does not depend on where the loader runs.
			this.addDependency(path.join(this.rootContext, "paths.json"));
			return "module.exports = require(" + JSON.stringify(require(path.join(this.rootContext, "paths.json")).api) + ");";
		case "imported.gen":
			return "import value from " + api + "; export default value;";
		case "dynamic.gen":
			return "module.exports = () => import(" + api + ");";
		case "removed.gen":
			return "if (module.hot) { var api = require(" + api + "); api.install(); } module.exports = 1;";
		case "data.gen":
			return "module.exports = [require(" + api + "), " + api + "];";
		case "shadowed.gen":
			return "module.exports = (function (require) { return require(" + api + "); })(value => value);";
		default:
			return source;
	}
};
`;

const REBASE = { rebaseLoaderRequests: true };

const GENERATED = ["required", "reexport", "keyed", "listed", "imported", "dynamic", "removed", "data", "shadowed"];

function writeProject(root) {
	const context = path.join(root, "app");
	fs.mkdirSync(path.join(root, ".git"), { recursive: true });
	fs.mkdirSync(path.join(context, "src"), { recursive: true });
	fs.mkdirSync(path.join(root, "node_modules/api"), { recursive: true });
	fs.writeFileSync(path.join(root, "node_modules/api/index.js"), "module.exports = 7;");
	fs.writeFileSync(path.join(context, "loader.cjs"), LOADER);
	fs.writeFileSync(
		path.join(context, "paths.json"),
		JSON.stringify({ api: path.join(root, "node_modules/api/index.js") })
	);
	for (const name of GENERATED) {
		fs.writeFileSync(path.join(context, "src", `${name}.gen`), name);
	}
	// Authored, so it names this exact file wherever the project moves to.
	fs.writeFileSync(
		path.join(context, "src/authored.js"),
		`module.exports = require(${JSON.stringify(path.join(root, "node_modules/api/index.js"))});`
	);
	fs.writeFileSync(
		path.join(context, "src/index.js"),
		`${[...GENERATED, "authored"]
			.map(name => `const ${name} = require("./${name}${name === "authored" ? ".js" : ".gen"}");`)
			.join("\n")}
module.exports = { required, reexport, keyed: keyed.value, listed, imported: imported.default, removed, data, shadowed, authored };`
	);
}

async function build(root, configuration = {}, portable = true) {
	const context = path.join(root, "app");
	globalThis[LOADER_RUNS] = [];
	const compiler = rspack({
		context,
		mode: "development",
		devtool: "source-map",
		target: "node",
		entry: "./src/index.js",
		output: {
			path: path.join(context, "dist"),
			library: { type: "commonjs2" },
			clean: true
		},
		cache: {
			type: "persistent",
			portable,
			storage: { type: "filesystem", location: path.join(context, ".cache") }
		},
		experiments: { portableCache: { seed: false } },
		module: {
			rules: [
				{
					test: /\.(gen|js)$/,
					include: path.join(context, "src"),
					type: "javascript/auto",
					use: [path.join(context, "loader.cjs")]
				}
			]
		},
		...configuration
	});
	try {
		const stats = await runCompiler(compiler);
		const { errors } = stats.toJson({ all: false, errors: true });
		expect(errors).toEqual([]);
		const dist = path.join(context, "dist");
		const output = fs.readFileSync(path.join(dist, "main.js"), "utf-8");
		const runtime = { exports: {} };
		new Function("module", "exports", "require", output)(
			runtime,
			runtime.exports,
			createRequire(path.join(dist, "main.js"))
		);
		return {
			values: runtime.exports,
			warnings: (
				stats.toJson({ all: false, logging: "warn" }).logging?.["rspack.persistentCache"]?.entries ?? []
			)
				.map(entry => entry.message)
				.filter(message => message.includes("rebaseLoaderRequests")),
			assets: fs
				.readdirSync(dist)
				.sort()
				.map(name => [name, fs.readFileSync(path.join(dist, name), "utf-8")]),
			runs: globalThis[LOADER_RUNS]
				.map(resource => path.relative(context, resource))
				.sort()
		};
	} finally {
		await closeCompiler(compiler);
		delete globalThis[LOADER_RUNS];
	}
}

const relocated = (temp, name, setup) => {
	const writer = path.join(temp, name, "writer", "proj");
	const reader = path.join(temp, name, "reader-with-longer-root", "proj");
	setup(writer);
	return { writer, reader };
};

const move = (writer, reader) => {
	fs.mkdirSync(path.dirname(reader), { recursive: true });
	fs.cpSync(writer, reader, { recursive: true, verbatimSymlinks: true });
};

const coldBuild = async (root, configuration, portable) => {
	fs.rmSync(path.join(root, "app/.cache"), { recursive: true, force: true });
	return build(root, configuration, portable);
};

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should reuse loader-generated absolute requests after a move and keep every other root path fixed",
	async run() {
		const temp = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-absolute-request-"))
		);
		try {
			const { writer, reader } = relocated(temp, "requests", writeProject);
			const first = await build(writer, {}, REBASE);
			expect(first.values.required).toBe(7);
			expect(first.values.data[1]).toBe(path.join(writer, "node_modules/api/index.js"));
			expect((await build(writer, {}, REBASE)).runs).toEqual([]);

			move(writer, reader);
			// Requests follow the moved checkout, the authored one stays with the writer.
			fs.writeFileSync(path.join(reader, "node_modules/api/index.js"), "module.exports = 8;");
			const moved = await build(reader, {}, REBASE);
			expect(moved.runs).toEqual([
				path.join("src", "authored.js"),
				path.join("src", "data.gen"),
				path.join("src", "keyed.gen"),
				path.join("src", "listed.gen"),
				path.join("src", "shadowed.gen")
			]);
			expect(moved.values).toEqual({
				required: 8,
				reexport: 8,
				keyed: 8,
				listed: 7,
				imported: 8,
				removed: 1,
				data: [8, path.join(reader, "node_modules/api/index.js")],
				shadowed: path.join(reader, "node_modules/api/index.js"),
				authored: 7
			});
			const cold = await coldBuild(reader, {}, REBASE);
			expect(moved.values).toEqual(cold.values);
			expect(moved.assets).toEqual(cold.assets);
			expect((await build(reader, {}, REBASE)).runs).toEqual([]);

			// Without the option a loader's path stays as written, so its modules are rebuilt.
			const notEnabled = relocated(temp, "not-enabled", writeProject);
			const expectWarning = ({ warnings }) => {
				expect(warnings).toHaveLength(1);
				expect(warnings[0]).toContain("7 in this build, such as");
				expect(warnings[0]).toContain("data.gen");
				expect(warnings[0]).toContain("cache.portable.rebaseLoaderRequests");
			};
			expectWarning(await build(notEnabled.writer));
			expect((await build(notEnabled.writer)).warnings).toEqual([]);
			move(notEnabled.writer, notEnabled.reader);
			const rebuilt = await build(notEnabled.reader);
			expectWarning(rebuilt);
			expect(rebuilt.runs).toEqual(
				[...GENERATED.map(name => `${name}.gen`), "authored.js"]
					.sort()
					.map(file => path.join("src", file))
			);
			const rebuiltCold = await coldBuild(notEnabled.reader);
			expect(rebuilt.assets).toEqual(rebuiltCold.assets);
			// A cached module holds one spelling, so switching the option starts a new cache.
			const switched = await build(notEnabled.reader, {}, REBASE);
			expect(switched.warnings).toEqual([]);
			const switchedCold = await coldBuild(notEnabled.reader, {}, REBASE);
			expect(switched.runs).toEqual(switchedCold.runs);
			expect(switched.assets).toEqual(switchedCold.assets);
			// The warning belongs to the experiment, and to projects where the option would apply.
			const noExperiment = await coldBuild(notEnabled.reader, { experiments: {} });
			expect(noExperiment.warnings).toEqual([]);
			const withExternals = await coldBuild(notEnabled.reader, {
				externals: /node_modules[\\/]api/,
				externalsType: "commonjs"
			});
			expect(withExternals.warnings).toEqual([]);

			// Options that read the absolute spelling keep it, so they behave as in any other build.
			const fixedSpelling = async (name, configure, expected) => {
				const project = relocated(temp, name, writeProject);
				const configuration = configure(project.writer);
				const before = await build(project.writer, configuration, REBASE);
				expect(before.values.required).toBe(expected);
				move(project.writer, project.reader);
				const after = await build(project.reader, configure(project.reader), REBASE);
				const uncached = await coldBuild(project.reader, configure(project.reader), REBASE);
				expect(after.values).toEqual(uncached.values);
				expect(after.assets).toEqual(uncached.assets);
				return after;
			};
			const external = await fixedSpelling(
				"externals",
				() => ({ externals: /node_modules[\\/]api/, externalsType: "commonjs" }),
				7
			);
			expect(external.assets.find(([name]) => name === "main.js")[1]).toContain(
				JSON.stringify(path.join(temp, "externals/reader-with-longer-root/proj/node_modules/api/index.js"))
			);
			// resolve.roots reads requests that start with "/", which an absolute Windows path does not.
			if (process.platform !== "win32") {
				await fixedSpelling(
					"roots",
					root => {
						const overlay = path.join(temp, "overlay", root);
						fs.mkdirSync(path.join(overlay, "node_modules/api"), { recursive: true });
						fs.writeFileSync(path.join(overlay, "node_modules/api/index.js"), "module.exports = 9;");
						return { resolve: { roots: [path.join(temp, "overlay")] } };
					},
					9
				);
			}

			await fixedSpelling(
				"alias-key",
				root => {
					fs.writeFileSync(path.join(root, "app/src/alias.js"), "module.exports = 9;");
					return {
						resolve: {
							alias: {
								[path.join(root, "node_modules/api/index.js")]: path.join(root, "app/src/alias.js")
							}
						}
					};
				},
				9
			);

			// A configuration that names a path in the writer still resolves there after a move.
			const hardCoded = async (name, configure, values) => {
				const project = relocated(temp, name, root => {
					writeProject(root);
					fs.writeFileSync(
						path.join(root, "app/src/index.js"),
						name === "provide"
							? "module.exports = { value: shimmed };"
							: name === "rule-loader"
								? 'module.exports = { value: require("./shim.js") };'
								: 'module.exports = { value: require("shim") };'
					);
					fs.writeFileSync(path.join(root, "app/src/shim.js"), "module.exports = 7;");
					fs.writeFileSync(path.join(root, "app/shim-loader.cjs"), 'module.exports = () => "module.exports = 7;";');
				});
				const configuration = configure(project.writer);
				await build(project.writer, configuration);
				move(project.writer, project.reader);
				fs.writeFileSync(path.join(project.reader, "app/src/shim.js"), "module.exports = 8;");
				fs.writeFileSync(path.join(project.reader, "app/shim-loader.cjs"), 'module.exports = () => "module.exports = 8;";');
				const after = await build(project.reader, configuration);
				expect(after.values).toEqual(values);
				const uncached = await coldBuild(project.reader, configuration);
				expect(after.values).toEqual(uncached.values);
				expect(after.assets).toEqual(uncached.assets);
			};
			await hardCoded(
				"replacement",
				root => ({
					plugins: [
						new rspack.NormalModuleReplacementPlugin(/^shim$/, path.join(root, "app/src/shim.js"))
					]
				}),
				{ value: 7 }
			);
			await hardCoded(
				"rule-loader",
				root => ({
					module: {
						rules: [{ test: /shim\.js$/, use: [path.join(root, "app/shim-loader.cjs")] }]
					}
				}),
				{ value: 7 }
			);
			await hardCoded(
				"provide",
				root => ({
					plugins: [new rspack.ProvidePlugin({ shimmed: path.join(root, "app/src/shim.js") })]
				}),
				{ value: 7 }
			);
			await hardCoded(
				"rule-alias",
				root => ({
					module: {
						rules: [
							{
								test: /index\.js$/,
								resolve: { alias: { shim: path.join(root, "app/src/shim.js") } }
							}
						]
					}
				}),
				{ value: 7 }
			);

			// A path that leaves the root does not move with it.
			const above = relocated(temp, "above", root => {
				writeProject(root);
				fs.mkdirSync(path.join(root, "../shared"), { recursive: true });
				fs.writeFileSync(path.join(root, "../shared/index.js"), "module.exports = 7;");
				fs.writeFileSync(
					path.join(root, "app/src/index.js"),
					'module.exports = { value: require("shared") };'
				);
			});
			const aboveAlias = root => ({
				resolve: { alias: { shared: `${root}/../shared/index.js` } }
			});
			await build(above.writer, aboveAlias(above.writer));
			move(above.writer, above.reader);
			fs.mkdirSync(path.join(above.reader, "../shared"), { recursive: true });
			fs.writeFileSync(path.join(above.reader, "../shared/index.js"), "module.exports = 8;");
			const aboveMoved = await build(above.reader, aboveAlias(above.reader));
			expect(aboveMoved.values).toEqual({ value: 8 });
			const aboveCold = await coldBuild(above.reader, aboveAlias(above.reader));
			expect(aboveMoved.assets).toEqual(aboveCold.assets);

			// A provided path computed for each checkout moves with it, so the module is reused.
			const provided = relocated(temp, "provided", root => {
				writeProject(root);
				fs.writeFileSync(path.join(root, "app/src/index.js"), "module.exports = { value: shimmed };");
				fs.writeFileSync(path.join(root, "app/src/shim.js"), "module.exports = 7;");
			});
			const provide = root => ({
				plugins: [new rspack.ProvidePlugin({ shimmed: path.join(root, "app/src/shim.js") })]
			});
			await build(provided.writer, provide(provided.writer));
			move(provided.writer, provided.reader);
			const providedMoved = await build(provided.reader, provide(provided.reader));
			expect(providedMoved.values).toEqual({ value: 7 });
			expect(providedMoved.runs).toEqual([]);
			const providedCold = await coldBuild(provided.reader, provide(provided.reader));
			expect(providedMoved.assets).toEqual(providedCold.assets);

			// A relative link moves with the checkout. One that points into the writer does not:
			// a cold build in the reader still follows it there.
			if (process.platform !== "win32") {
				const links = relocated(temp, "links", root => {
					writeProject(root);
					for (const name of ["relative", "absolute"]) {
						fs.mkdirSync(path.join(root, "packages", name), { recursive: true });
						fs.writeFileSync(
							path.join(root, "packages", name, "index.js"),
							"module.exports = 7;"
						);
					}
					fs.symlinkSync("../packages/relative", path.join(root, "node_modules/relative"));
					fs.symlinkSync(
						path.join(root, "packages/absolute"),
						path.join(root, "node_modules/absolute")
					);
					fs.writeFileSync(
						path.join(root, "app/src/index.js"),
						`module.exports = { relative: require("relative"), absolute: require("absolute") };`
					);
				});
				await build(links.writer);
				move(links.writer, links.reader);
				for (const name of ["relative", "absolute"]) {
					fs.writeFileSync(
						path.join(links.reader, "packages", name, "index.js"),
						"module.exports = 8;"
					);
				}
				const linked = await build(links.reader);
				expect(linked.values).toEqual({ relative: 8, absolute: 7 });
				expect(linked.runs).toEqual([]);
				const linkedCold = await coldBuild(links.reader);
				expect(linked.values).toEqual(linkedCold.values);
				expect(linked.assets).toEqual(linkedCold.assets);

				// A worktree that links node_modules to the main checkout resolves into that checkout.
				const shared = relocated(temp, "shared-node-modules", writeProject);
				await build(shared.writer);
				move(shared.writer, shared.reader);
				fs.rmSync(path.join(shared.reader, "node_modules"), { recursive: true });
				fs.symlinkSync(path.join(shared.writer, "node_modules"), path.join(shared.reader, "node_modules"));
				const sharedMoved = await build(shared.reader);
				const sharedCold = await coldBuild(shared.reader);
				expect(sharedMoved.values).toEqual(sharedCold.values);
				expect(sharedMoved.assets).toEqual(sharedCold.assets);
			}
		} finally {
			fs.rmSync(temp, { recursive: true, force: true });
		}
	}
};
