const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { rspack } = require("@rspack/core");

const fixtureRoot = path.resolve(__dirname, "js/chunk-graph-entry-options");

function write(root, filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	return file;
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

function evaluateOrder(compilation, filename) {
	const context = { globalThis: { order: [] } };
	vm.runInNewContext(
		compilation.getAsset(filename).source.source().toString(),
		context
	);
	return context.globalThis.order;
}

function originRequests(compilation, chunkName) {
	return compilation
		.getStats()
		.toJson({ all: false, chunks: true, chunkOrigins: true })
		.chunks.find(chunk => chunk.names.includes(chunkName))
		.origins.map(origin => origin.request);
}

function codeSplittingMessages(compilation) {
	return (
		compilation.getStats().toJson({ all: false, logging: "verbose" }).logging?.[
			"rspack.Compilation.codeSplittingCache"
		]?.entries ?? []
	).map(entry => entry.message);
}

async function close(compiler) {
	await new Promise((resolve, reject) => {
		compiler.close(error => (error ? reject(error) : resolve()));
	});
}

describe("incremental chunk graph entry roots", () => {
	afterAll(() => {
		fs.rmSync(fixtureRoot, { force: true, recursive: true });
	});

	it("rebuilds the chunk graph when dynamic entry options change", async () => {
		const root = path.join(fixtureRoot, "filename");
		fs.rmSync(root, { force: true, recursive: true });
		write(root, "src/index.js", "import './leaf';\n");
		write(root, "src/leaf.js", "export const value = 'before';\n");
		let filename = "before.js";
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			entry: () => ({ main: { import: "./src/index.js", filename } }),
			output: { path: path.join(root, "dist"), filename: "[name].js" },
			optimization: { minimize: false, splitChunks: false }
		});

		try {
			const initial = await run(compiler);
			expect(initial.getAsset("before.js")).toBeDefined();
			filename = "after.js";
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);

			expect(updated.getAsset("after.js")).toBeDefined();
			expect(updated.getAsset("before.js")).toBeUndefined();
		} finally {
			await close(compiler);
		}
	});

	it("keeps dynamic entry import targets and execution order current", async () => {
		const root = path.join(fixtureRoot, "imports");
		fs.rmSync(root, { force: true, recursive: true });
		write(
			root,
			"src/a.js",
			"import './a-child'; globalThis.order.push('A');\n"
		);
		write(
			root,
			"src/b.js",
			"import './b-child'; globalThis.order.push('B');\n"
		);
		write(root, "src/a-child.js", "export const value = 'a';\n");
		write(root, "src/b-child.js", "export const value = 'b';\n");
		write(root, "src/leaf.js", "export const value = 'before';\n");
		let imports = ["./src/a.js", "./src/b.js"];
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			entry: () => ({
				keeper: { import: ["./src/a.js", "./src/b.js", "./src/leaf.js"] },
				main: { import: imports }
			}),
			output: { path: path.join(root, "dist"), filename: "[name].js" },
			optimization: { minimize: false, splitChunks: false }
		});

		try {
			const initial = await run(compiler);
			expect(evaluateOrder(initial, "main.js")).toEqual(["A", "B"]);
			imports = ["./src/b.js", "./src/a.js"];
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);

			expect(evaluateOrder(updated, "main.js")).toEqual(["B", "A"]);
		} finally {
			await close(compiler);
		}
	});

	it("keeps unnamed global entry targets current when another entry retains both modules", async () => {
		const root = path.join(fixtureRoot, "global-entry");
		fs.rmSync(root, { force: true, recursive: true });
		write(
			root,
			"src/a.js",
			"import './a-child'; globalThis.order.push('A');\n"
		);
		write(
			root,
			"src/b.js",
			"import './b-child'; globalThis.order.push('B');\n"
		);
		write(root, "src/a-child.js", "export const value = 'a';\n");
		write(root, "src/b-child.js", "export const value = 'b';\n");
		write(root, "src/main.js", "globalThis.order.push('M');\n");
		write(root, "src/leaf.js", "export const value = 'before';\n");
		let globalEntry = "./src/a.js";
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			entry: {
				keeper: { import: ["./src/a.js", "./src/b.js", "./src/leaf.js"] },
				main: { import: "./src/main.js" }
			},
			output: { path: path.join(root, "dist"), filename: "[name].js" },
			optimization: { minimize: false, splitChunks: false },
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.make.tapPromise(
							"global-entry-regression",
							compilation => {
								return new Promise((resolve, reject) => {
									compilation.addEntry(
										compiler.context,
										rspack.EntryPlugin.createDependency(globalEntry),
										{},
										error => (error ? reject(error) : resolve())
									);
								});
							}
						);
					}
				}
			]
		});

		try {
			const initial = await run(compiler);
			expect(evaluateOrder(initial, "main.js")).toEqual(["A", "M"]);
			globalEntry = "./src/b.js";
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);

			expect(evaluateOrder(updated, "main.js")).toEqual(["B", "M"]);
		} finally {
			await close(compiler);
		}
	});

	it("keeps dynamic entry request attribution current when spelling changes", async () => {
		const root = path.join(fixtureRoot, "origin-request");
		fs.rmSync(root, { force: true, recursive: true });
		write(
			root,
			"src/a.js",
			"import './a-child'; globalThis.order.push('A');\n"
		);
		write(root, "src/a-child.js", "export const value = 'a';\n");
		write(root, "src/leaf.js", "export const value = 'before';\n");
		let entryRequest = "./src/a.js";
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			entry: () => ({
				keeper: { import: ["./src/a.js", "./src/leaf.js"] },
				main: { import: entryRequest }
			}),
			output: { path: path.join(root, "dist"), filename: "[name].js" },
			optimization: { minimize: false, splitChunks: false }
		});

		try {
			const initial = await run(compiler);
			expect(originRequests(initial, "main")).toContain("./src/a.js");
			entryRequest = "./src/./a.js";
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n"
			);
			const updated = await rebuild(compiler, [leaf]);

			expect(originRequests(updated, "main")).toContain("./src/./a.js");
			expect(originRequests(updated, "main")).not.toContain("./src/a.js");
		} finally {
			await close(compiler);
		}
	});

	it("reuses an unchanged chunk graph when a global and named entry share a root", async () => {
		const root = path.join(fixtureRoot, "duplicate-root");
		fs.rmSync(root, { force: true, recursive: true });
		write(
			root,
			"src/a.js",
			"import './a-child'; globalThis.order.push('A');\n",
		);
		write(root, "src/a-child.js", "export const value = 'a';\n");
		write(root, "src/leaf.js", "export const value = 'before';\n");
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			entry: {
				keeper: { import: "./src/leaf.js" },
				main: { import: "./src/a.js" },
			},
			output: { path: path.join(root, "dist"), filename: "[name].js" },
			optimization: { minimize: false, splitChunks: false },
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.make.tapPromise(
							"duplicate-root-regression",
							compilation => {
								return new Promise((resolve, reject) => {
									compilation.addEntry(
										compiler.context,
										rspack.EntryPlugin.createDependency("./src/a.js"),
										{},
										error => (error ? reject(error) : resolve()),
									);
								});
							},
						);
					},
				},
			],
		});

		try {
			await run(compiler);
			const leaf = write(
				root,
				"src/leaf.js",
				"export const value = 'after';\n",
			);
			const updated = await rebuild(compiler, [leaf]);

			expect(codeSplittingMessages(updated)).toEqual([]);
		} finally {
			await close(compiler);
		}
	});

	it.each([
		["named", { name: "main" }],
		["global", {}]
	])(
		"keeps %s include roots current when another entry retains both modules",
		async (kind, options) => {
			const root = path.join(fixtureRoot, `${kind}-include`);
			fs.rmSync(root, { force: true, recursive: true });
			write(
				root,
				"src/a.js",
				"import './a-child'; globalThis.order.push('A');\n"
			);
			write(
				root,
				"src/b.js",
				"import './b-child'; globalThis.order.push('B');\n"
			);
			write(root, "src/a-child.js", "export const value = 'a';\n");
			write(root, "src/b-child.js", "export const value = 'b';\n");
			write(root, "src/main.js", "globalThis.order.push('M');\n");
			write(root, "src/leaf.js", "export const value = 'before';\n");
			let included = "./src/a.js";
			const compiler = rspack({
				context: root,
				mode: "development",
				target: "node",
				devtool: false,
				cache: true,
				incremental: true,
				entry: {
					keeper: { import: ["./src/a.js", "./src/b.js", "./src/leaf.js"] },
					main: { import: "./src/main.js" }
				},
				output: { path: path.join(root, "dist"), filename: "[name].js" },
				optimization: { minimize: false, splitChunks: false },
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.finishMake.tapPromise(
								`${kind}-include-regression`,
								compilation =>
									new Promise((resolve, reject) => {
										compilation.addInclude(
											compiler.context,
											rspack.EntryPlugin.createDependency(included),
											options,
											error => (error ? reject(error) : resolve())
										);
									})
							);
						}
					}
				]
			});

			try {
				const initial = await run(compiler);
				const initialSource = initial
					.getAsset("main.js")
					.source.source()
					.toString();
				expect(initialSource).toContain('"./src/a.js"');
				expect(initialSource).not.toContain('"./src/b.js"');
				included = "./src/b.js";
				const leaf = write(
					root,
					"src/leaf.js",
					"export const value = 'after';\n"
				);
				const updated = await rebuild(compiler, [leaf]);
				const updatedSource = updated
					.getAsset("main.js")
					.source.source()
					.toString();

				expect(updatedSource).toContain('"./src/b.js"');
				expect(updatedSource).not.toContain('"./src/a.js"');
			} finally {
				await close(compiler);
			}
		}
	);

	it.each(["shared", "captured"])(
		"refreshes %s dynamic filename and publicPath functions while reusing the chunk graph",
		async functionMode => {
			const root = path.join(fixtureRoot, `function-${functionMode}`);
			fs.rmSync(root, { force: true, recursive: true });
			write(
				root,
				"src/main.js",
				"import(/* webpackChunkName: 'lazy' */ './lazy'); globalThis.main = true;\n"
			);
			write(root, "src/lazy.js", "export const value = 'lazy';\n");
			write(
				root,
				"src/leaf.js",
				"import './leaf-child'; export const value = 'before';\n"
			);
			write(root, "src/leaf-child.js", "export const child = true;\n");
			let filenameValue = "before-main.js";
			let publicPathValue = "/before-assets/";
			const sharedFilename = () => filenameValue;
			const sharedPublicPath = () => publicPathValue;
			const compiler = rspack({
				context: root,
				mode: "development",
				target: "web",
				devtool: false,
				cache: true,
				experiments: { cache: true },
				incremental: true,
				entry: () => {
					const capturedFilename = filenameValue;
					const capturedPublicPath = publicPathValue;
					return {
						keeper: { import: "./src/leaf.js" },
						main: {
							import: "./src/main.js",
							filename:
								functionMode === "shared"
									? sharedFilename
									: () => capturedFilename,
							publicPath:
								functionMode === "shared"
									? sharedPublicPath
									: () => capturedPublicPath
						}
					};
				},
				output: {
					path: path.join(root, "dist"),
					filename: "[name].js",
					chunkFilename: "[name].js"
				},
				optimization: {
					minimize: false,
					splitChunks: false,
					moduleIds: "named",
					chunkIds: "named"
				}
			});

			try {
				const initial = await run(compiler);
				expect(initial.getAsset("before-main.js")).toBeDefined();
				expect(
					initial.getAsset("before-main.js").source.source().toString()
				).toContain("/before-assets/");

				let leaf = write(
					root,
					"src/leaf.js",
					"import './leaf-child'; export const value = 'stable-edit';\n"
				);
				const stable = await rebuild(compiler, [leaf]);
				expect(codeSplittingMessages(stable)).toEqual([]);
				expect(stable.getAsset("before-main.js")).toBeDefined();

				filenameValue = "after-main.js";
				publicPathValue = "/after-assets/";
				leaf = write(
					root,
					"src/leaf.js",
					"import './leaf-child'; export const value = 'changed-edit';\n"
				);
				const updated = await rebuild(compiler, [leaf]);
				const updatedSource = updated
					.getAsset("after-main.js")
					.source.source()
					.toString();

				expect(codeSplittingMessages(updated)).toEqual([]);
				expect(updated.getAsset("before-main.js")).toBeUndefined();
				expect(updatedSource).toContain("/after-assets/");
				expect(updatedSource).not.toContain("/before-assets/");
			} finally {
				await close(compiler);
			}
		}
	);
});
