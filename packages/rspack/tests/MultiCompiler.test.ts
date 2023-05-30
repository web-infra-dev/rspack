import path from "path";
import { createFsFromVolume, Volume } from "memfs";
import { FileSystemInfoEntry, Watcher } from "../src/util/fs";
import { Compiler, MultiRspackOptions, rspack, RspackOptions } from "../src";
import { assert } from "console";

const createMultiCompiler = (
	options?: RspackOptions[] | { parallelism?: number }
) => {
	const compiler = rspack(
		Object.assign(
			[
				{
					name: "a",
					context: path.join(__dirname, "fixtures"),
					entry: "./a.js"
				},
				{
					name: "b",
					context: path.join(__dirname, "fixtures"),
					entry: "./b.js"
				}
			],
			options
		)
	);
	compiler.outputFileSystem = createFsFromVolume(new Volume());
	compiler.watchFileSystem = {
		watch(a, b, c, d, e, f, g) {
			return null as any;
		}
	};
	return compiler;
};

describe("MultiCompiler", function () {
	jest.setTimeout(20000);

	it("should trigger 'run' for each child compiler", done => {
		const compiler = createMultiCompiler();
		let called = 0;

		compiler.hooks.run.tap("MultiCompiler test", () => called++);
		compiler.run(err => {
			if (err) {
				throw err;
			}
			expect(called).toBe(2);
			compiler.close(done);
		});
	});
	it("should trigger 'watchRun' for each child compiler", done => {
		const compiler = createMultiCompiler();
		let called = 0;

		compiler.hooks.watchRun.tap("MultiCompiler test", () => called++);
		compiler.watch({ aggregateTimeout: 1000 }, err => {
			if (err) {
				throw err;
			}
			expect(called).toBe(2);
			compiler.close(done);
		});
	});
	// Running this will cause a segmentation fault in `RwLock` of compiler/mod.rs:119, more diagnostics are necessary.
	it.skip("should not be running twice at a time (run)", done => {
		const compiler = createMultiCompiler();
		compiler.run((err, stats) => {
			if (err) return done(err);
		});
		compiler.run((err, stats) => {
			if (err) {
				compiler.close(done);
			}
		});
	});
	// Running this will cause a segmentation fault in `RwLock` of compiler/mod.rs:119, more diagnostics are necessary.
	it.skip("should not be running twice at a time (watch)", done => {
		const compiler = createMultiCompiler();
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		compiler.watch({}, (err, stats) => {
			if (err) {
				compiler.close(done);
			}
		});
	});
	// Running this will cause a segmentation fault in `RwLock` of compiler/mod.rs:119, more diagnostics are necessary.
	it.skip("should not be running twice at a time (run - watch)", done => {
		const compiler = createMultiCompiler();
		compiler.run((err, stats) => {
			if (err) return done(err);
		});
		compiler.watch({}, (err, stats) => {
			if (err) {
				compiler.close(done);
			}
		});
	});
	// Running this will cause a segmentation fault in `RwLock` of compiler/mod.rs:119, more diagnostics are necessary.
	it.skip("should not be running twice at a time (watch - run)", done => {
		const compiler = createMultiCompiler();
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		compiler.run((err, stats) => {
			if (err) {
				compiler.close(done);
			}
		});
	});
	// Running this will cause a segmentation fault in `RwLock` of compiler/mod.rs:119, more diagnostics are necessary.
	it.skip("should not be running twice at a time (instance cb)", done => {
		const compiler = rspack(
			{
				context: __dirname,
				mode: "production",
				entry: "./c",
				output: {
					path: "/",
					filename: "bundle.js"
				}
			},
			() => {}
		);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		compiler.run((err, stats) => {
			if (err) {
				compiler.close(done);
			}
		});
	});
	it("should run again correctly after first compilation", done => {
		const compiler = createMultiCompiler();
		compiler.run((err, stats) => {
			if (err) return done(err);

			compiler.run((err, stats) => {
				if (err) return done(err);
				compiler.close(done);
			});
		});
	});
	it("should watch again correctly after first compilation", done => {
		const compiler = createMultiCompiler();
		compiler.run((err, stats) => {
			if (err) return done(err);

			compiler.watch({}, (err, stats) => {
				if (err) return done(err);
				compiler.close(done);
			});
		});
	});
	it("should run again correctly after first closed watch", done => {
		const compiler = createMultiCompiler();
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		watching!.close(() => {
			compiler.run((err, stats) => {
				if (err) return done(err);
				compiler.close(done);
			});
		});
	});
	it("should watch again correctly after first closed watch", done => {
		const compiler = createMultiCompiler();
		const watching = compiler.watch({}, (err, stats) => {
			if (err) return done(err);
		});
		watching!.close(() => {
			compiler.watch({}, (err, stats) => {
				if (err) return done(err);
				compiler.close(done);
			});
		});
	});
	it("should respect parallelism and dependencies for running", done => {
		const compiler = createMultiCompiler({
			parallelism: 1,
			2: {
				name: "c",
				context: path.join(__dirname, "fixtures"),
				entry: "./a.js",
				dependencies: ["d", "e"]
			},
			3: {
				name: "d",
				context: path.join(__dirname, "fixtures"),
				entry: "./a.js"
			},
			4: {
				name: "e",
				context: path.join(__dirname, "fixtures"),
				entry: "./a.js"
			}
		});
		const events: string[] = [];
		compiler.compilers.forEach(c => {
			c.hooks.run.tap("test", () => {
				events.push(`${c.name} run`);
			});
			c.hooks.done.tap("test", () => {
				events.push(`${c.name} done`);
			});
		});
		compiler.run((err, stats) => {
			expect(events.join(" ")).toBe(
				"a run a done b run b done d run d done e run e done c run c done"
			);
			compiler.close(done);
		});
	});
	// Parse error: CJS Top level return
	it.skip("should respect parallelism and dependencies for watching", done => {
		const compiler = rspack(
			Object.assign(
				[
					{
						name: "a",
						mode: "development" as const,
						context: path.join(__dirname, "fixtures"),
						entry: "./a.js",
						dependencies: ["b", "c"]
					},
					{
						name: "b",
						mode: "development" as const,
						context: path.join(__dirname, "fixtures"),
						entry: "./b.js"
					},
					{
						name: "c",
						mode: "development" as const,
						context: path.join(__dirname, "fixtures"),
						entry: "./a.js"
					}
				],
				{ parallelism: 1 }
			)
		);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watchCallbacks: ((
			error: Error | null,
			fileTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			contextTimeInfoEntries: Map<string, FileSystemInfoEntry | "ignore">,
			changedFiles: Set<string>,
			removedFiles: Set<string>
		) => void)[] = [];
		const watchCallbacksUndelayed: (() => void)[] = [];
		compiler.watchFileSystem = {
			watch(
				files,
				directories,
				missing,
				startTime,
				options,
				callback,
				callbackUndelayed
			) {
				watchCallbacks.push(callback);
				watchCallbacksUndelayed.push(callbackUndelayed as any);
				return null as any;
			}
		};
		const events: string[] = [];
		compiler.compilers.forEach(c => {
			c.hooks.invalid.tap("test", () => {
				events.push(`${c.name} invalid`);
			});
			c.hooks.watchRun.tap("test", () => {
				events.push(`${c.name} run`);
			});
			c.hooks.done.tap("test", () => {
				events.push(`${c.name} done`);
			});
		});

		let update = 0;
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
			const info = () => stats!.toString({ preset: "summary", version: false });
			switch (update++) {
				case 0:
					expect(info()).toMatchInlineSnapshot(`
							"a:
							  a compiled successfully

							b:
							  b compiled successfully

							c:
							  c compiled successfully"
					`);
					expect(compiler.compilers[0].modifiedFiles).toBe(undefined);
					expect(compiler.compilers[0].removedFiles).toBe(undefined);
					expect(events).toMatchInlineSnapshot(`
				[
				  "b run",
				  "b done",
				  "c run",
				  "c done",
				  "a run",
				  "a done",
				]
			`);
					events.length = 0;
					// wait until watching begins
					setTimeout(() => {
						watchCallbacksUndelayed[0]();
						watchCallbacks[0](null, new Map(), new Map(), new Set(), new Set());
					}, 100);
					break;
				case 1:
					expect(info()).toMatchInlineSnapshot(`
				"a:
				  a compiled successfully

				b:
				  b compiled successfully"
			`);
					expect(compiler.compilers[1].modifiedFiles).toEqual(new Set());
					expect(compiler.compilers[1].removedFiles).toEqual(new Set());
					expect(events).toMatchInlineSnapshot(`
				[
				  "b invalid",
				  "b run",
				  "b done",
				  "a invalid",
				  "a run",
				  "a done",
				]
			`);
					watchCallbacksUndelayed[2]();
					watchCallbacks[2](null, new Map(), new Map(), new Set(), new Set());
					break;
				case 2:
					expect(info()).toMatchInlineSnapshot(`
				"a:
				  a compiled successfully"
			`);
					expect(events).toMatchInlineSnapshot(`
				[
				  "b invalid",
				  "b run",
				  "b done",
				  "a invalid",
				  "a run",
				  "a done",
				  "a invalid",
				  "a run",
				  "a done",
				]
			`);
					events.length = 0;
					watchCallbacksUndelayed[0]();
					watchCallbacksUndelayed[1]();
					watchCallbacks[0](null, new Map(), new Map(), new Set(), new Set());
					watchCallbacks[1](null, new Map(), new Map(), new Set(), new Set());
					break;
				case 3:
					expect(info()).toMatchInlineSnapshot(`
				"a:
				  a compiled successfully

				b:
				  b compiled successfully

				c:
				  c compiled successfully"
			`);
					expect(events).toMatchInlineSnapshot(`
				[
				  "b invalid",
				  "c invalid",
				  "b run",
				  "b done",
				  "c run",
				  "c done",
				  "a invalid",
				  "a run",
				  "a done",
				]
			`);
					events.length = 0;
					compiler.close(done);
					break;
				default:
					done(new Error("unexpected"));
			}
		});
	});
	it("should respect parallelism when using invalidate", done => {
		const configs: MultiRspackOptions = [
			{
				name: "a",
				mode: "development",
				entry: { a: "./a.js" },
				context: path.join(__dirname, "fixtures")
			},
			{
				name: "b",
				mode: "development",
				entry: { b: "./b.js" },
				context: path.join(__dirname, "fixtures")
			}
		];
		configs.parallelism = 1;
		const compiler = rspack(configs);

		const events: string[] = [];
		compiler.compilers.forEach(c => {
			c.hooks.invalid.tap("test", () => {
				events.push(`${c.name} invalid`);
			});
			c.hooks.watchRun.tap("test", () => {
				events.push(`${c.name} run`);
			});
			c.hooks.done.tap("test", () => {
				events.push(`${c.name} done`);
			});
		});

		compiler.outputFileSystem = createFsFromVolume(new Volume());

		let state = 0;
		const watching = compiler.watch({}, error => {
			if (error) {
				done(error);
				return;
			}
			if (state !== 0) return;
			state++;

			expect(events).toMatchInlineSnapshot(`
			[
			  "a run",
			  "a done",
			  "b run",
			  "b done",
			]
		`);
			events.length = 0;

			watching.invalidate(err => {
				try {
					if (err) return done(err);

					expect(events).toMatchInlineSnapshot(`
				[
				  "a invalid",
				  "b invalid",
				  "a run",
				  "a done",
				  "b run",
				  "b done",
				]
			`);
					events.length = 0;
					expect(state).toBe(1);
					setTimeout(() => {
						compiler.close(done);
					}, 1000);
				} catch (e) {
					console.error(e);
					done(e);
				}
			});
		});
	}, 20000);

	// issue #2585
	it("should respect parallelism when using watching", done => {
		const configMaps: any = [];

		for (let index = 0; index < 3; index++) {
			configMaps.push({
				name: index.toString(),
				mode: "development",
				entry: "./src/main.jsx",
				devServer: {
					hot: true
				}
			});
		}
		configMaps.parallelism = 1;
		const compiler = rspack(configMaps);

		compiler.watch({}, err => {
			if (err) {
				done(err);
			} else {
				done();
			}
		});
	}, 20000);

	it("should respect dependencies when using invalidate", done => {
		const compiler = rspack([
			{
				name: "a",
				mode: "development",
				entry: { a: "./a.js" },
				context: path.join(__dirname, "fixtures"),
				dependencies: ["b"]
			},
			{
				name: "b",
				mode: "development",
				entry: { b: "./b.js" },
				context: path.join(__dirname, "fixtures")
			}
		]);

		const events: string[] = [];
		compiler.compilers.forEach(c => {
			c.hooks.invalid.tap("test", () => {
				events.push(`${c.name} invalid`);
			});
			c.hooks.watchRun.tap("test", () => {
				events.push(`${c.name} run`);
			});
			c.hooks.done.tap("test", () => {
				events.push(`${c.name} done`);
			});
		});

		// @ts-ignore
		compiler.watchFileSystem = { watch() {} };
		compiler.outputFileSystem = createFsFromVolume(new Volume());

		let state = 0;
		const watching = compiler.watch({}, error => {
			if (error) {
				done(error);
				return;
			}
			if (state !== 0) return;
			state++;

			expect(events).toMatchInlineSnapshot(`
			[
			  "b run",
			  "b done",
			  "a run",
			  "a done",
			]
		`);
			events.length = 0;

			watching.invalidate(err => {
				try {
					if (err) return done(err);

					expect(events).toMatchInlineSnapshot(`
				[
				  "a invalid",
				  "b invalid",
				  "b run",
				  "b done",
				  "a run",
				  "a done",
				]
			`);
					events.length = 0;
					expect(state).toBe(1);
					setTimeout(() => {
						compiler.close(done);
					}, 1000);
				} catch (e) {
					console.error(e);
					done(e);
				}
			});
		});
	}, 20000);

	it("shouldn't hang when invalidating watchers", done => {
		const entriesA: { a: string; b?: string } = { a: "./a.js" };
		const entriesB: { a?: string; b: string } = { b: "./b.js" };
		const compiler = rspack([
			{
				name: "a",
				mode: "development",
				entry: entriesA,
				context: path.join(__dirname, "fixtures")
			},
			{
				name: "b",
				mode: "development",
				entry: entriesB,
				context: path.join(__dirname, "fixtures")
			}
		]);

		// @ts-ignore
		compiler.watchFileSystem = { watch() {} };
		compiler.outputFileSystem = createFsFromVolume(new Volume());

		const watching = compiler.watch({}, error => {
			if (error) {
				done(error);
				return;
			}

			entriesA.b = "./b.js";
			entriesB.a = "./a.js";

			watching.invalidate(err => {
				if (err) return done(err);
				compiler.close(done);
			});
		});
	}, 20000);

	it("shouldn't hang when invalidating during build", done => {
		const compiler = rspack(
			Object.assign([
				{
					name: "a",
					mode: "development",
					context: path.join(__dirname, "fixtures"),
					entry: "./a.js"
				},
				{
					name: "b",
					mode: "development",
					context: path.join(__dirname, "fixtures"),
					entry: "./b.js",
					dependencies: ["a"]
				}
			])
		);
		compiler.outputFileSystem = createFsFromVolume(new Volume());
		const watchCallbacks: any[] = [];
		const watchCallbacksUndelayed: ((
			fileName: string,
			changeTime: number
		) => void)[] = [];
		let firstRun = true;
		compiler.watchFileSystem = {
			watch(
				files,
				directories,
				missing,
				startTime,
				options,
				callback,
				callbackUndelayed
			) {
				watchCallbacks.push(callback);
				watchCallbacksUndelayed.push(callbackUndelayed);

				const filesSet = new Set(files);
				if (
					firstRun &&
					filesSet.has(path.join(__dirname, "fixtures", "a.js"))
				) {
					process.nextTick(() => {
						callback(null, new Map(), new Map(), new Set(), new Set());
					});
					firstRun = false;
				}
				return null as unknown as Watcher;
			}
		};
		compiler.watch({}, (err, stats) => {
			if (err) return done(err);
			compiler.close(done);
		});
	}, 20000);
});

describe.skip("Pressure test", function () {
	it("should work well in multiCompilers", done => {
		const configs = Array(100).fill({
			context: path.join(__dirname, "fixtures"),
			entry: "./a.js"
		});

		const multiCompiler = rspack(configs);

		multiCompiler.run(err => {
			if (err) done(err);
			else done();
		});
	});

	it("should work well in concurrent", async () => {
		const total = 100;

		let finish = 0;

		const runnings: Promise<null>[] = [];

		for (let i = 0; i < total; i++) {
			if (i % 10 == 0) {
				// Insert new instance while we are running
				rspack(
					{
						context: path.join(__dirname, "fixtures"),
						entry: "./a.js"
					},
					() => {}
				);
			}

			runnings.push(
				new Promise(resolve => {
					rspack(
						{
							context: path.join(__dirname, "fixtures"),
							entry: "./a.js"
						},
						err => {
							resolve(null);
							if (!err) finish++;
						}
					);
				})
			);
		}

		await Promise.all(runnings);
		expect(finish).toBe(total);
	});
});
