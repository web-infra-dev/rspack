const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { rspack, lazyCompilationMiddleware } = require("@rspack/core");

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

function watchOnce(compiler) {
	let watching;
	const result = new Promise((resolve, reject) => {
		const timeout = setTimeout(
			() => reject(new Error("Timed out waiting for a lazy compilation")),
			10000
		);
		watching = compiler.watch({}, (error, stats) => {
			clearTimeout(timeout);
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve(stats.compilation);
		});
	});
	return { result, watching };
}

function moduleIdentifiers(compilation) {
	return [...compilation.modules].map(module => module.identifier());
}

async function close(compiler) {
	await new Promise((resolve, reject) => {
		compiler.close(error => (error ? reject(error) : resolve()));
	});
}

describe("binding entry dependency identity", () => {
	it("keeps unnamed entries and named/global includes eager during lazy compilation", async () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-binding-lazy-"));
		write(root, "src/main.js", "globalThis.named = 'NAMED_ENTRY';\n");
		write(root, "src/global.js", "globalThis.global = 'GLOBAL_ENTRY';\n");
		write(
			root,
			"src/named-include.js",
			"globalThis.namedInclude = 'NAMED_INCLUDE';\n"
		);
		write(
			root,
			"src/global-include.js",
			"globalThis.globalInclude = 'GLOBAL_INCLUDE';\n"
		);
		const compiler = rspack({
			context: root,
			mode: "development",
			target: "web",
			devtool: false,
			cache: true,
			incremental: true,
			entry: { main: "./src/main.js" },
			lazyCompilation: { entries: true, imports: false },
			output: {
				path: path.join(root, "dist"),
				filename: "main.js",
				chunkFilename: "[name].js"
			},
			optimization: {
				minimize: false,
				splitChunks: false,
				moduleIds: "named",
				chunkIds: "named"
			},
			plugins: [
				{
					apply(currentCompiler) {
						currentCompiler.hooks.make.tapPromise(
							"binding-global-entry",
							compilation =>
								new Promise((resolve, reject) => {
									compilation.addEntry(
										currentCompiler.context,
										rspack.EntryPlugin.createDependency("./src/global.js"),
										{},
										error => (error ? reject(error) : resolve())
									);
								})
						);
						currentCompiler.hooks.finishMake.tapPromise(
							"binding-includes",
							compilation =>
								Promise.all(
									[
										["./src/named-include.js", { name: "main" }],
										["./src/global-include.js", {}]
									].map(
										([request, options]) =>
											new Promise((resolve, reject) => {
												compilation.addInclude(
													currentCompiler.context,
													rspack.EntryPlugin.createDependency(request),
													options,
													error => (error ? reject(error) : resolve())
												);
											})
									)
								)
						);
					}
				}
			]
		});
		lazyCompilationMiddleware(compiler);
		const { result, watching } = watchOnce(compiler);

		try {
			const compilation = await result;
			const modules = moduleIdentifiers(compilation);
			const source = compilation.getAsset("main.js").source.source().toString();
			const isProxyFor = filename =>
				modules.some(
					identifier =>
						identifier.includes("lazy-compilation-proxy") &&
						identifier.includes(filename)
				);

			expect(isProxyFor("global.js")).toBe(false);
			expect(isProxyFor("named-include.js")).toBe(false);
			expect(isProxyFor("global-include.js")).toBe(false);
			expect(isProxyFor("main.js")).toBe(true);
			expect(source).toContain("GLOBAL_ENTRY");
			expect(source).toContain("NAMED_INCLUDE");
			expect(source).toContain("GLOBAL_INCLUDE");
			expect(source).not.toContain("NAMED_ENTRY");
		} finally {
			await new Promise((resolve, reject) =>
				watching.close(error => (error ? reject(error) : resolve()))
			);
			fs.rmSync(root, { force: true, recursive: true });
		}
	});

	it.each([
		["addEntry", "make", "entry.js"],
		["addInclude", "finishMake", "include.js"]
	])(
		"keeps the raw context in the %s dependency cache key",
		async (method, hook, request) => {
			const root = fs.mkdtempSync(
				path.join(os.tmpdir(), `rspack-binding-${method}-`)
			);
			const contextA = path.join(root, "a");
			const contextB = path.join(root, "b");
			write(root, "src/main.js", "globalThis.main = true;\n");
			write(contextA, request, "globalThis.fromA = 'CONTEXT_A_PAYLOAD';\n");
			write(contextB, request, "globalThis.fromB = 'CONTEXT_B_PAYLOAD';\n");
			const compiler = rspack({
				context: root,
				mode: "development",
				target: "node",
				devtool: false,
				cache: true,
				incremental: true,
				entry: { main: "./src/main.js" },
				output: { path: path.join(root, "dist"), filename: "main.js" },
				optimization: {
					minimize: false,
					splitChunks: false,
					moduleIds: "named"
				},
				plugins: [
					{
						apply(currentCompiler) {
							currentCompiler.hooks[hook].tapPromise(
								`binding-${method}-context`,
								compilation =>
									Promise.all(
										[contextA, contextB].map(
											context =>
												new Promise((resolve, reject) => {
													compilation[method](
														context,
														rspack.EntryPlugin.createDependency(`./${request}`),
														{ name: "main" },
														error => (error ? reject(error) : resolve())
													);
												})
										)
									)
							);
						}
					}
				]
			});

			try {
				const compilation = await run(compiler);
				const modules = moduleIdentifiers(compilation).join("\n");
				const source = compilation
					.getAsset("main.js")
					.source.source()
					.toString();

				expect(modules).toContain(path.join("a", request));
				expect(modules).toContain(path.join("b", request));
				expect(source).toContain("CONTEXT_A_PAYLOAD");
				expect(source).toContain("CONTEXT_B_PAYLOAD");
			} finally {
				await close(compiler);
				fs.rmSync(root, { force: true, recursive: true });
			}
		}
	);
});
