const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const rspack = require("@rspack/core");

const names = ["source", "dependent", "tail", "unrelated"];
const parent = {
	source: undefined,
	dependent: "source",
	tail: "dependent",
	unrelated: undefined
};

async function withTimeout(promise, description) {
	let timeout;
	try {
		return await Promise.race([
			promise,
			new Promise((_, reject) => {
				timeout = setTimeout(
					() => reject(new Error(`Timed out waiting for ${description}`)),
					15000
				);
			})
		]);
	} finally {
		clearTimeout(timeout);
	}
}

describe("lazy MultiCompiler artifact provenance", () => {
	async function runArtifactChain(_watcherName, nativeWatcher) {
		const root = await fs.mkdtemp(path.join(os.tmpdir(), "rspack-lazy-chain-"));
		const sourceFile = path.join(root, "source.js");
		const coalescedEntry = path.join(root, "coalesced-source.js");
		const stampFile = name => path.join(root, name, "stamp.txt");
		const generations = Object.fromEntries(names.map(name => [name, 0]));
		const events = [];
		const invalidations = [];
		let delayedSource;
		let resolveFileInvalidation;
		let server;
		let watching;

		try {
			await Promise.all(
				names.map(name =>
					fs.writeFile(
						path.join(root, `${name}.js`),
						`export const ${name} = "${name}";\n`
					)
				)
			);
			await fs.writeFile(
				coalescedEntry,
				'export const coalesced = "coalesced";\n'
			);
			// Keep Watchpack's initial scan from treating fresh fixture files as edits.
			const beforeWatch = new Date(Date.now() - 3000);
			await Promise.all(
				[
					...names.map(name => path.join(root, `${name}.js`)),
					coalescedEntry
				].map(file => fs.utimes(file, beforeWatch, beforeWatch))
			);

			const config = name => ({
				context: root,
				dependencies: parent[name] ? [parent[name]] : [],
				devtool: false,
				entry:
					name === "source"
						? { main: "./source.js", coalesced: "./coalesced-source.js" }
						: { main: `./${name}.js` },
				experiments: { nativeWatcher },
				lazyCompilation: { entries: true, imports: false },
				mode: "development",
				name,
				output: {
					chunkFilename: "[name].js",
					filename: "[name].js",
					path: path.join(root, name)
				},
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.invalid.tap("lazy-artifact-chain", file => {
								invalidations.push({ name, file });
								if (name === "source" && file === sourceFile) {
									resolveFileInvalidation?.();
								}
							});
							compiler.hooks.make.tapPromise(
								"lazy-artifact-chain",
								async () => {
									if (name === "source" && delayedSource) {
										delayedSource.started();
										await delayedSource.release;
									}
								}
							);
							compiler.hooks.thisCompilation.tap(
								"lazy-artifact-chain",
								compilation => {
									compilation.hooks.processAssets.tapPromise(
										{
											name: "lazy-artifact-chain",
											stage:
												compilation.constructor.PROCESS_ASSETS_STAGE_ADDITIONS
										},
										async () => {
											generations[name] += 1;
											const upstream = parent[name]
												? (
														await fs.readFile(stampFile(parent[name]), "utf8")
													).trim()
												: "";
											const stamp = upstream
												? `${upstream}>${name}${generations[name]}`
												: `${name}${generations[name]}`;
											compilation.emitAsset(
												"stamp.txt",
												new rspack.sources.RawSource(`${stamp}\n`)
											);
										}
									);
								}
							);
							compiler.hooks.done.tap("lazy-artifact-chain", stats => {
								events.push({
									name,
									kind: stats.compilation.watchInvalidationKind,
									changed: [...(compiler.modifiedFiles ?? [])].map(file =>
										path.basename(file)
									)
								});
							});
						}
					}
				],
				target: "web"
			});

			const compiler = rspack.rspack(names.map(config));
			const middleware = rspack.lazyCompilationMiddleware(compiler);
			server = http.createServer((request, response) =>
				middleware(request, response, () => {
					response.statusCode = 404;
					response.end();
				})
			);
			await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
			const { port } = server.address();

			const builds = [];
			const waiters = [];
			const nextBuild = () =>
				builds.length
					? Promise.resolve(builds.shift())
					: new Promise((resolve, reject) => {
							const timeout = setTimeout(
								() => reject(new Error("Timed out waiting for a watch build")),
								15000
							);
							waiters.push(value => {
								clearTimeout(timeout);
								resolve(value);
							});
						});
			watching = compiler.watch({ aggregateTimeout: 20 }, (error, stats) => {
				const value = {
					error:
						error ??
						(stats?.hasErrors()
							? new Error(stats.toString({ all: false, errors: true }))
							: undefined),
					children: stats?.stats ?? [stats]
				};
				(waiters.shift() ?? (result => builds.push(result)))(value);
			});

			const assertBuild = (build, expectedNames, expectedKind) => {
				assert.equal(build.error, undefined);
				const children = build.children.filter(Boolean);
				assert.deepEqual(
					children.map(child => child.compilation.name).sort(),
					[...expectedNames].sort()
				);
				assert.deepEqual(
					children.map(child => child.compilation.watchInvalidationKind),
					children.map(child =>
						typeof expectedKind === "function"
							? expectedKind(child.compilation.name)
							: expectedKind
					)
				);
			};
			const assertTail = async () => {
				assert.equal(
					(await fs.readFile(stampFile("tail"), "utf8")).trim(),
					`source${generations.source}>dependent${generations.dependent}>tail${generations.tail}`
				);
			};

			assertBuild(await nextBuild(), names, undefined);
			const baseline = { ...generations };
			await assertTail();
			const lazyIds = new Map();
			for (const name of names) {
				const bundle = await fs.readFile(
					path.join(root, name, "main.js"),
					"utf8"
				);
				const encoded = bundle.match(/var data = ("(?:[^"\\]|\\.)*")/)?.[1];
				assert.notEqual(
					encoded,
					undefined,
					`Missing lazy module ID for ${name}`
				);
				lazyIds.set(name, JSON.parse(encoded));
			}
			const coalescedBundle = await fs.readFile(
				path.join(root, "source", "coalesced.js"),
				"utf8"
			);
			const coalescedId = coalescedBundle.match(
				/var data = ("(?:[^"\\]|\\.)*")/
			)?.[1];
			assert.notEqual(
				coalescedId,
				undefined,
				"Missing coalesced lazy module ID"
			);
			const activate = (name, moduleId = lazyIds.get(name)) =>
				new Promise((resolve, reject) => {
					const request = http.request(
						{
							host: "127.0.0.1",
							method: "POST",
							path: `/_rspack/lazy/trigger__${names.indexOf(name)}`,
							port
						},
						response => {
							response.resume();
							response.on("end", () => resolve(response.statusCode));
						}
					);
					request.on("error", reject);
					request.end(moduleId);
				});

			const beforeUnrelated = events.length;
			assert.equal(await activate("unrelated"), 200);
			assertBuild(await nextBuild(), ["unrelated"], "lazy");
			assert.deepEqual(
				events.slice(beforeUnrelated).map(event => event.name),
				["unrelated"]
			);
			assert.equal(generations.source, baseline.source);
			assert.equal(generations.dependent, baseline.dependent);
			assert.equal(generations.tail, baseline.tail);

			const beforeLazy = events.length;
			assert.equal(await activate("source"), 200);
			assertBuild(await nextBuild(), ["source", "dependent", "tail"], name =>
				name === "source" ? "lazy" : "normal"
			);
			assert.deepEqual(
				events.slice(beforeLazy).map(event => [event.name, event.kind]),
				[
					["source", "lazy"],
					["dependent", "normal"],
					["tail", "normal"]
				]
			);
			await assertTail();

			let signalSourceStarted;
			let releaseSource;
			const sourceStarted = new Promise(resolve => {
				signalSourceStarted = resolve;
			});
			const sourceRelease = new Promise(resolve => {
				releaseSource = resolve;
			});
			const fileInvalidated = new Promise(resolve => {
				resolveFileInvalidation = resolve;
			});
			delayedSource = {
				started: signalSourceStarted,
				release: sourceRelease
			};
			const beforeCoalesced = events.length;
			const beforeGenerations = { ...generations };
			assert.equal(await activate("source", JSON.parse(coalescedId)), 200);
			await withTimeout(sourceStarted, "the coalesced source build to start");
			const beforeFileInvalidations = invalidations.length;
			await fs.writeFile(
				sourceFile,
				'export const source = "source";\nexport const edited = "file-edit";\n'
			);
			if (nativeWatcher) {
				await withTimeout(fileInvalidated, "the coalesced source file event");
			}
			releaseSource();
			assertBuild(await nextBuild(), ["source", "dependent", "tail"], "normal");
			const coalesced = events.slice(beforeCoalesced);
			assert.equal(
				invalidations
					.slice(beforeFileInvalidations)
					.filter(event => event.name === "source").length,
				1,
				"a coalesced file edit must notify the source compiler exactly once"
			);
			assert.equal(
				coalesced.every(event => event.kind === "normal"),
				true,
				`normal must dominate a coalesced file edit: ${JSON.stringify(coalesced)}`
			);
			assert.equal(
				coalesced.some(
					event =>
						event.name === "source" && event.changed.includes("source.js")
				),
				true,
				`expected a file-backed source generation: ${JSON.stringify(coalesced)}`
			);
			const emittedJavaScript = await Promise.all(
				(await fs.readdir(path.join(root, "source")))
					.filter(file => file.endsWith(".js"))
					.map(file => fs.readFile(path.join(root, "source", file), "utf8"))
			);
			assert.equal(
				emittedJavaScript.some(source => source.includes("file-edit")),
				true,
				"the coalesced file edit must reach an emitted JavaScript asset"
			);
			assert.ok(generations.source > beforeGenerations.source);
			assert.equal(generations.dependent, beforeGenerations.dependent + 1);
			assert.equal(generations.tail, beforeGenerations.tail + 1);
			assert.equal(generations.unrelated, beforeGenerations.unrelated);
			await assertTail();
			const deliveredGenerations = { ...generations };
			await new Promise(resolve => setTimeout(resolve, 80));
			assert.deepEqual(generations, deliveredGenerations);
		} finally {
			if (watching) {
				await new Promise((resolve, reject) =>
					watching.close(error => (error ? reject(error) : resolve()))
				);
			}
			if (server) await new Promise(resolve => server.close(resolve));
			await fs.rm(root, { force: true, recursive: true });
		}
	}

	it.each([
		["native", true],
		["watchpack", false]
	])(
		"keeps dependent and coalesced file-backed generations normal with %s watching",
		runArtifactChain
	);
});
