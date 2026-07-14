const fs = require("node:fs");
const path = require("node:path");
const { CopyRspackPlugin, Stats, rspack } = require("@rspack/core");

const fixtureRoot = path.resolve(__dirname, "js/copy-plugin-cache");

function write(root, filename, content) {
	const file = path.join(root, filename);
	fs.mkdirSync(path.dirname(file), { recursive: true });
	fs.writeFileSync(file, content);
	return file;
}

function remove(root, filename) {
	const file = path.join(root, filename);
	fs.rmSync(file);
	return file;
}

function createCompiler(name, patterns, nativeWatcher = false, prepare) {
	const root = path.join(fixtureRoot, name);
	fs.rmSync(root, { recursive: true, force: true });
	write(root, "src/index.js", "module.exports = 'initial';\n");
	prepare?.(root);

	return {
		root,
		compiler: rspack({
			context: root,
			mode: "development",
			target: "node",
			devtool: false,
			cache: true,
			incremental: true,
			experiments: { nativeWatcher },
			entry: "./src/index.js",
			output: { path: path.join(root, "dist"), filename: "main.js" },
			plugins: [new CopyRspackPlugin({ patterns })]
		})
	};
}

function compile(compiler) {
	return new Promise((resolve, reject) => {
		compiler.run((error, stats) => {
			if (error) return reject(error);
			if (stats.hasErrors()) {
				return reject(new Error(stats.toString({ all: false, errors: true })));
			}
			resolve(compiler._lastCompilation);
		});
	});
}

function rebuild(compiler, modifiedFiles = [], removedFiles = []) {
	return new Promise((resolve, reject) => {
		compiler.__internal__rebuild(
			new Set(modifiedFiles),
			new Set(removedFiles),
			error => {
				if (error) return reject(error);
				const compilation = compiler._lastCompilation;
				if (compilation.errors.length > 0) {
					return reject(
						new Error(new Stats(compilation).toString({ all: false, errors: true }))
					);
				}
				resolve(compilation);
			}
		);
	});
}

function rebuildAllowErrors(compiler, modifiedFiles = [], removedFiles = []) {
	return new Promise((resolve, reject) => {
		compiler.__internal__rebuild(
			new Set(modifiedFiles),
			new Set(removedFiles),
			error => {
				if (error) return reject(error);
				resolve(compiler._lastCompilation);
			}
		);
	});
}

function watch(compiler) {
	const builds = [];
	const waiters = [];
	const watching = compiler.watch({ aggregateTimeout: 0 }, (error, stats) => {
		const build = { error, stats };
		const waiter = waiters.shift();
		if (waiter) waiter(build);
		else builds.push(build);
	});

	return {
		next(timeoutMs = 5000) {
			if (builds.length > 0) return Promise.resolve(builds.shift());
			return new Promise((resolve, reject) => {
				const timeout = setTimeout(
					() => reject(new Error("timed out waiting for a copy-plugin rebuild")),
					timeoutMs
				);
				waiters.push(build => {
					clearTimeout(timeout);
					resolve(build);
				});
			});
		},
		close() {
			return new Promise((resolve, reject) => {
				watching.close(error => (error ? reject(error) : resolve()));
			});
		}
	};
}

function asset(compilation, filename) {
	return compilation.getAsset(filename)?.source.source().toString();
}

async function nextBuildWithAsset(watching, filename) {
	const deadline = Date.now() + 5000;

	while (Date.now() < deadline) {
		const build = await watching.next(deadline - Date.now());
		if (build.error || asset(build.stats.compilation, filename) !== undefined) {
			return build;
		}
	}

	throw new Error(`timed out waiting for copied asset '${filename}'`);
}

function reusedPatterns(compilation) {
	const logging = new Stats(compilation).toJson({
		all: false,
		logging: false,
		loggingDebug: [/CopyRspackPlugin/]
	}).logging;

	return Object.values(logging || {})
		.flatMap(group => group.entries || [])
		.filter(entry =>
			entry.message?.startsWith("reusing unchanged copy pattern")
		).length;
}

async function close(compiler) {
	await new Promise((resolve, reject) => {
		compiler.close(error => (error ? reject(error) : resolve()));
	});
}

describe("CopyRspackPlugin pattern cache", () => {
	afterAll(() => {
		fs.rmSync(fixtureRoot, { recursive: true, force: true });
	});

	it.each([
		["watchpack", false],
		["native watcher", true]
	])(
		"watches the stable glob base and copies a newly populated sibling with %s",
		async (watcherName, nativeWatcher) => {
			const { root, compiler } = createCompiler(
				`glob-sibling-${watcherName.replace(" ", "-")}`,
				[{ from: "assets/*/*.txt", to: "copied", toType: "dir" }],
				nativeWatcher,
				root => {
					write(root, "assets/a/one.txt", "one\n");
					fs.mkdirSync(path.join(root, "assets/b"), { recursive: true });
				}
			);
			const watching = watch(compiler);

			try {
				const initial = await watching.next();
				expect(initial.error).toBeNull();
				expect(initial.stats.hasErrors()).toBe(false);
				expect([...initial.stats.compilation.contextDependencies]).toContain(
					path.join(root, "assets")
				);
				expect(asset(initial.stats.compilation, "copied/assets/a/one.txt")).toBe(
					"one\n"
				);

				await new Promise(resolve => setTimeout(resolve, 200));
				write(root, "assets/b/two.txt", "two\n");
				const sibling = await nextBuildWithAsset(
					watching,
					"copied/assets/b/two.txt"
				);

				expect(sibling.error).toBeNull();
				expect(sibling.stats.hasErrors()).toBe(false);
				expect(asset(sibling.stats.compilation, "copied/assets/b/two.txt")).toBe(
					"two\n"
				);

				write(root, "src/index.js", "module.exports = 'changed';\n");
				const updated = await watching.next();

				expect(updated.error).toBeNull();
				expect(updated.stats.hasErrors()).toBe(false);
				expect(asset(updated.stats.compilation, "copied/assets/a/one.txt")).toBe(
					"one\n"
				);
				expect(asset(updated.stats.compilation, "copied/assets/b/two.txt")).toBe(
					"two\n"
				);
			} finally {
				await watching.close();
			}
		}
	);

	it.each([
		["directory", "assets/nested", "copied-dir/one.txt"],
		["file", "assets/nested/one.txt", "copied-file/one.txt"]
	])("invalidates a %s pattern for an ancestor-directory event", async (kind, from, emitted) => {
		const { root, compiler } = createCompiler(`ancestor-${kind}`, [
			{ from, to: kind === "file" ? "copied-file" : "copied-dir", toType: "dir" }
		]);
		write(root, "assets/nested/one.txt", "before\n");

		try {
			const initial = await compile(compiler);
			expect(asset(initial, emitted)).toBe("before\n");

			write(root, "assets/nested/one.txt", "after\n");
			const updated = await rebuild(compiler, [path.join(root, "assets")]);

			expect(asset(updated, emitted)).toBe("after\n");
			expect(reusedPatterns(updated)).toBe(0);
		} finally {
			await close(compiler);
		}
	});

	it("invalidates modified and removed children, including removing the last glob match and re-adding it", async () => {
		const { root, compiler } = createCompiler("glob-add-remove", [
			{
				from: "assets/glob/**/*.txt",
				to: "copied",
				toType: "dir",
				noErrorOnMissing: true,
				globOptions: { ignore: ["**/skip-*.txt"] }
			}
		]);
		const one = write(root, "assets/glob/nested/one.txt", "one\n");

		try {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/assets/glob/nested/one.txt")).toBe("one\n");

			const two = write(root, "assets/glob/nested/two.txt", "two\n");
			let updated = await rebuild(compiler, [two]);
			expect(asset(updated, "copied/assets/glob/nested/two.txt")).toBe("two\n");

			write(root, "assets/glob/nested/two.txt", "two-edited\n");
			updated = await rebuild(compiler, [two]);
			expect(asset(updated, "copied/assets/glob/nested/two.txt")).toBe("two-edited\n");

			remove(root, "assets/glob/nested/one.txt");
			remove(root, "assets/glob/nested/two.txt");
			updated = await rebuild(compiler, [], [one, two]);
			expect(asset(updated, "copied/assets/glob/nested/one.txt")).toBeUndefined();
			expect(asset(updated, "copied/assets/glob/nested/two.txt")).toBeUndefined();

			const three = write(root, "assets/glob/other/three.txt", "three\n");
			const ignored = write(root, "assets/glob/other/skip-three.txt", "ignored\n");
			updated = await rebuild(compiler, [three, ignored]);
			expect(asset(updated, "copied/assets/glob/other/three.txt")).toBe("three\n");
			expect(asset(updated, "copied/assets/glob/other/skip-three.txt")).toBeUndefined();
		} finally {
			await close(compiler);
		}
	});

	it("reuses unchanged static patterns but never caches callback, template, or permission patterns", async () => {
		let transformCalls = 0;
		let destinationCalls = 0;
		const { root, compiler } = createCompiler("cache-policy", [
			{ from: "assets/static", to: "static" },
			{
				from: "assets/transform",
				to: "transform",
				transform(content) {
					transformCalls += 1;
					return content;
				}
			},
			{
				from: "assets/function",
				to({ absoluteFilename }) {
					destinationCalls += 1;
					return `function/${path.basename(absoluteFilename)}`;
				}
			},
			{ from: "assets/template", to: "template/[name][ext]" },
			{ from: "assets/permissions", to: "permissions", copyPermissions: true }
		]);
		write(root, "assets/static/one.txt", "static\n");
		write(root, "assets/transform/one.txt", "transform\n");
		write(root, "assets/function/one.txt", "function\n");
		write(root, "assets/template/one.txt", "template\n");
		write(root, "assets/permissions/one.txt", "permissions\n");

		try {
			await compile(compiler);
			expect(transformCalls).toBe(1);
			expect(destinationCalls).toBe(1);

			const entry = write(root, "src/index.js", "module.exports = 'changed';\n");
			const updated = await rebuild(compiler, [entry]);

			expect(reusedPatterns(updated)).toBe(1);
			expect(transformCalls).toBe(2);
			expect(destinationCalls).toBe(2);
			expect(asset(updated, "static/one.txt")).toBe("static\n");
			expect(asset(updated, "transform/one.txt")).toBe("transform\n");
			expect(asset(updated, "function/one.txt")).toBe("function\n");
			expect(asset(updated, "template/one.txt")).toBe("template\n");
			expect(asset(updated, "permissions/one.txt")).toBe("permissions\n");
		} finally {
			await close(compiler);
		}
	});

	it("does not reuse cached results across compiler.run calls without watcher provenance", async () => {
		const { root, compiler } = createCompiler("run-twice", [
			{ from: "assets/source", to: "copied" }
		]);
		write(root, "assets/source/one.txt", "before\n");

		try {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/one.txt")).toBe("before\n");

			write(root, "assets/source/one.txt", "after\n");
			const updated = await compile(compiler);

			expect(asset(updated, "copied/one.txt")).toBe("after\n");
			expect(reusedPatterns(updated)).toBe(0);
		} finally {
			await close(compiler);
		}
	});

	it("evicts a cached pattern before an errored recomputation and recovers when the source returns", async () => {
		const { root, compiler } = createCompiler("diagnostic-recovery", [
			{ from: "assets/source/*.txt", to: "copied", toType: "dir" }
		]);
		const source = write(root, "assets/source/one.txt", "before\n");

		try {
			const initial = await compile(compiler);
			expect(asset(initial, "copied/assets/source/one.txt")).toBe("before\n");

			remove(root, "assets/source/one.txt");
			let failed = await rebuildAllowErrors(compiler, [], [source]);
			expect(failed.errors.length).toBeGreaterThan(0);
			expect(
				failed.errors.filter(error =>
					String(error.message ?? error).includes("unable to locate")
				)
			).toHaveLength(1);
			expect(reusedPatterns(failed)).toBe(0);
			expect(asset(failed, "copied/assets/source/one.txt")).toBeUndefined();

			const entry = write(root, "src/index.js", "module.exports = 'changed';\n");
			failed = await rebuildAllowErrors(compiler, [entry]);
			expect(failed.errors.length).toBeGreaterThan(0);
			expect(reusedPatterns(failed)).toBe(0);
			expect(asset(failed, "copied/assets/source/one.txt")).toBeUndefined();

			write(root, "assets/source/one.txt", "after\n");
			const recovered = await rebuild(compiler, [source]);
			expect(asset(recovered, "copied/assets/source/one.txt")).toBe("after\n");
			expect(reusedPatterns(recovered)).toBe(0);
		} finally {
			await close(compiler);
		}
	});
});
