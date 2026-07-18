const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { rspack, lazyCompilationMiddleware } = require("@rspack/core");

describe("lazy MultiCompiler prefix routing", () => {
	it("activates compiler 10 without matching the compiler 1 prefix", async () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-lazy-prefix-"));
		const names = Array.from({ length: 11 }, (_, index) => `compiler-${index}`);
		for (const name of names) {
			fs.writeFileSync(
				path.join(root, `${name}.js`),
				`export const name = '${name}';\n`
			);
		}

		const builds = [];
		const waiters = [];
		const nextBuild = () =>
			builds.length > 0
				? Promise.resolve(builds.shift())
				: new Promise((resolve, reject) => {
						const timeout = setTimeout(
							() => reject(new Error("Timed out waiting for a lazy rebuild")),
							10000
						);
						waiters.push(value => {
							clearTimeout(timeout);
							resolve(value);
						});
					});
		const compiler = rspack(
			names.map(name => ({
				context: root,
				mode: "development",
				name,
				target: "web",
				devtool: false,
				entry: `./${name}.js`,
				lazyCompilation: { entries: true, imports: false },
				output: {
					path: path.join(root, name),
					filename: "main.js",
					chunkFilename: "[name].js"
				}
			}))
		);
		const middleware = lazyCompilationMiddleware(compiler);
		const watching = compiler.watch({}, (error, stats) => {
			const value = {
				error:
					error ??
					(stats?.hasErrors()
						? new Error(stats.toString({ all: false, errors: true }))
						: undefined),
				stats
			};
			(waiters.shift() ?? (build => builds.push(build)))(value);
		});

		try {
			const initial = await nextBuild();
			expect(initial.error).toBeUndefined();
			const bundle = fs.readFileSync(
				path.join(root, "compiler-10", "main.js"),
				"utf8"
			);
			const encoded = bundle.match(/var data = ("(?:[^"\\]|\\.)*")/)?.[1];
			expect(encoded).toBeDefined();
			const moduleId = JSON.parse(encoded);

			await new Promise((resolve, reject) => {
				middleware(
					{
						body: [moduleId],
						method: "POST",
						url: "/_rspack/lazy/trigger__10?source=test"
					},
					{
						end: resolve,
						write() {},
						writeHead(status) {
							expect(status).toBe(200);
						}
					},
					reject
				).catch(reject);
			});
			const updated = await nextBuild();
			expect(updated.error).toBeUndefined();
			expect(updated.stats.stats.map(child => child.compilation.name)).toEqual([
				"compiler-10"
			]);
			expect(updated.stats.stats[0].compilation.watchInvalidationKind).toBe(
				"lazy"
			);
		} finally {
			await new Promise((resolve, reject) =>
				watching.close(error => (error ? reject(error) : resolve()))
			);
			fs.rmSync(root, { force: true, recursive: true });
		}
	});
});
