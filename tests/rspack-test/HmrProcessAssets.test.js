const fs = require("node:fs");
const path = require("node:path");
const { rspack } = require("@rspack/core");

const fixtureRoot = path.resolve(__dirname, "js/hmr-process-assets");

function write(root, filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	return file;
}

function createCompiler(
	name,
	files,
	optimization = { splitChunks: false },
	incremental = true
) {
	const root = path.join(fixtureRoot, name);
	fs.rmSync(root, { recursive: true, force: true });
	for (const [filename, content] of Object.entries(files)) {
		write(root, filename, content);
	}

	return {
		root,
		compiler: rspack({
			context: root,
			mode: "development",
			target: "web",
			devtool: false,
			cache: true,
			incremental,
			entry: { a: "./src/a.js", b: "./src/b.js" },
			output: {
				path: path.join(root, "dist"),
				filename: "[name].js",
				chunkFilename: "[name].js"
			},
			optimization,
			plugins: [new rspack.HotModuleReplacementPlugin()]
		})
	};
}

function run(compiler) {
	return new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve(stats.compilation);
		});
	});
}

function rebuild(compiler, modifiedFiles) {
	return new Promise((resolve, reject) => {
		compiler.__internal__rebuild(new Set(modifiedFiles), new Set(), error => {
			if (error) return reject(error);
			const compilation = compiler._lastCompilation;
			if (compilation.errors.length > 0) {
				return reject(new Error(compilation.errors.join("\n")));
			}
			resolve(compilation);
		});
	});
}

function hotAssets(compilation) {
	return compilation
		.getAssets()
		.filter(({ name }) => name.includes("hot-update"));
}

function hotManifests(compilation) {
	return hotAssets(compilation)
		.filter(({ name }) => name.endsWith(".json"))
		.map(({ name, source }) => ({
			name,
			...JSON.parse(source.source().toString())
		}));
}

async function close(compiler) {
	await new Promise((resolve, reject) => {
		compiler.close(error => (error ? reject(error) : resolve()));
	});
}

describe("HotModuleReplacementPlugin process assets", () => {
	afterAll(() => {
		fs.rmSync(fixtureRoot, { recursive: true, force: true });
	});

	it("emits a changed shared module for every stable runtime", async () => {
		const entry = name =>
			`import { value } from './leaf'; globalThis.${name} = value; if (module.hot) module.hot.accept('./leaf');\n`;
		const { root, compiler } = createCompiler("stable-runtimes", {
			"src/a.js": entry("a"),
			"src/b.js": entry("b"),
			"src/leaf.js": "export const value = 'before';\n"
		});

		try {
			await run(compiler);
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);
			const manifests = hotManifests(updated);
			const hotJavaScript = hotAssets(updated).filter(({ name }) =>
				name.endsWith(".js")
			);

			expect(manifests).toHaveLength(2);
			expect(manifests.every(({ c }) => c.length > 0)).toBe(true);
			expect(
				manifests.every(({ r, m }) => r.length === 0 && m.length === 0)
			).toBe(true);
			expect(hotJavaScript).toHaveLength(2);
			expect(
				hotJavaScript.every(({ source }) =>
					source.source().toString().includes("after")
				)
			).toBe(true);
		} finally {
			await close(compiler);
		}
	});

	it("preserves a shared async chunk until its final runtime is removed", async () => {
		const entry = (name, includeShared) =>
			`${
				includeShared
					? `import(/* webpackChunkName: 'shared' */ './shared').then(({ value }) => { globalThis.${name} = value; });`
					: `globalThis.${name} = 'removed';`
			}\nif (module.hot) module.hot.accept();\n`;
		const { root, compiler } = createCompiler("removed-runtime", {
			"src/a.js": entry("a", true),
			"src/b.js": entry("b", true),
			"src/shared.js": "export const value = 'shared';\n"
		});

		try {
			await run(compiler);
			const a = write(root, "src/a.js", entry("a", false));
			const first = await rebuild(compiler, [a]);
			const firstManifests = hotManifests(first);
			const removedManifest = firstManifests.find(({ name }) =>
				name.startsWith("a.")
			);
			const activeManifest = firstManifests.find(({ name }) =>
				name.startsWith("b.")
			);
			expect(firstManifests).toHaveLength(2);
			expect(first.getAsset("shared.js")).toBeDefined();
			expect(removedManifest).toBeDefined();
			expect(removedManifest.r).toContain("shared");
			expect(removedManifest.m).toContain("./src/shared.js");
			expect(activeManifest).toBeDefined();
			expect(activeManifest.r).not.toContain("shared");
			expect(activeManifest.m).not.toContain("./src/shared.js");

			const b = write(root, "src/b.js", entry("b", false));
			const last = await rebuild(compiler, [b]);
			const manifests = hotManifests(last);

			expect(last.getAsset("shared.js")).toBeUndefined();
			expect(manifests.some(({ r }) => r.includes("shared"))).toBe(true);
			expect(manifests.some(({ m }) => m.includes("./src/shared.js"))).toBe(
				true
			);
		} finally {
			await close(compiler);
		}
	});

	it("updates an installed chunk when an edited module moves within a stable runtime", async () => {
		const entry = (name, includeMoving) =>
			`import { anchor } from './shared/anchor';\n${
				includeMoving ? "import { moving } from './shared/moving';\n" : ""
			}globalThis.${name} = anchor${includeMoving ? " + moving" : ""};\nif (module.hot) module.hot.accept();\n`;
		const { root, compiler } = createCompiler(
			"moved-module",
			{
				"src/a.js": entry("a", true),
				"src/b.js": entry("b", true),
				"src/shared/anchor.js": "export const anchor = 'anchor';\n",
				"src/shared/moving.js": "export const moving = 'moving-before';\n"
			},
			{
				runtimeChunk: "single",
				splitChunks: {
					chunks: "all",
					minSize: 0,
					cacheGroups: {
						shared: {
							test: /[\\/]shared[\\/]/,
							name: "shared",
							minChunks: 2,
							enforce: true
						}
					}
				}
			}
		);

		try {
			const initial = await run(compiler);
			expect(
				initial.getAsset("shared.js").source.source().toString()
			).toContain("moving-before");

			const b = write(root, "src/b.js", entry("b", false));
			const moving = write(
				root,
				"src/shared/moving.js",
				"export const moving = 'moving-after';\n"
			);
			const updated = await rebuild(compiler, [b, moving]);
			expect(updated.getAsset("a.js").source.source().toString()).toContain(
				"moving-after"
			);
			expect(
				updated.getAsset("shared.js").source.source().toString()
			).not.toContain("moving-after");

			const hotWithMarker = hotAssets(updated)
				.filter(
					({ name, source }) =>
						name.endsWith(".js") &&
						source.source().toString().includes("moving-after")
				)
				.map(({ name }) => name);
			expect(hotWithMarker.some(name => name.startsWith("a."))).toBe(true);
			expect(hotWithMarker.some(name => name.startsWith("shared."))).toBe(true);
		} finally {
			await close(compiler);
		}
	});

	it("falls back to a full HMR scan when incremental chunk assets are disabled", async () => {
		const entry = name =>
			`import { value } from './leaf'; globalThis.${name} = value; if (module.hot) module.hot.accept('./leaf');\n`;
		const { root, compiler } = createCompiler(
			"chunk-asset-disabled",
			{
				"src/a.js": entry("a"),
				"src/b.js": entry("b"),
				"src/leaf.js": "export const value = 'before';\n"
			},
			{ splitChunks: false },
			{ chunkAsset: false }
		);

		try {
			await run(compiler);
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);
			const hotJavaScript = hotAssets(updated).filter(({ name }) =>
				name.endsWith(".js")
			);

			expect(hotManifests(updated)).toHaveLength(2);
			expect(hotJavaScript).toHaveLength(2);
			expect(
				hotJavaScript.every(({ source }) =>
					source.source().toString().includes("after")
				)
			).toBe(true);
		} finally {
			await close(compiler);
		}
	});
});
