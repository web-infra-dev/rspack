const path = require("path");
const fs = require("fs");

const rspack = require("@rspack/core");

const { run, runEmit, runChange } = require("./helpers/run");

const { readAssets, getCompiler, compile } = require("./helpers");

const FIXTURES_DIR = path.join(__dirname, "fixtures");

describe("CopyPlugin", () => {
	describe("basic", () => {
		it("should copy a file", done => {
			runEmit({
				expectedAssetKeys: ["file.txt"],
				patterns: [
					{
						from: "file.txt"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files", done => {
			runEmit({
				expectedAssetKeys: [
					".dottedfile",
					"directoryfile.txt",
					"nested/deep-nested/deepnested.txt",
					"nested/nestedfile.txt"
				],
				patterns: [
					{
						from: "directory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files to new directory", done => {
			runEmit({
				expectedAssetKeys: [
					"newdirectory/.dottedfile",
					"newdirectory/directoryfile.txt",
					"newdirectory/nested/deep-nested/deepnested.txt",
					"newdirectory/nested/nestedfile.txt"
				],
				patterns: [
					{
						from: "directory",
						to: "newdirectory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files to new directory with context", done => {
			runEmit({
				expectedAssetKeys: [
					"newdirectory/deep-nested/deepnested.txt",
					"newdirectory/nestedfile.txt"
				],
				patterns: [
					{
						from: "nested",
						context: "directory",
						to: "newdirectory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files using glob", done => {
			runEmit({
				expectedAssetKeys: [
					"directory/directoryfile.txt",
					"directory/nested/deep-nested/deepnested.txt",
					"directory/nested/nestedfile.txt"
				],
				patterns: [
					{
						from: "directory/**/*"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files using glob to new directory", done => {
			runEmit({
				expectedAssetKeys: [
					"newdirectory/directory/directoryfile.txt",
					"newdirectory/directory/nested/deep-nested/deepnested.txt",
					"newdirectory/directory/nested/nestedfile.txt"
				],
				patterns: [
					{
						from: "directory/**/*",
						to: "newdirectory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy files using glob to new directory with context", done => {
			runEmit({
				expectedAssetKeys: [
					"newdirectory/nested/deep-nested/deepnested.txt",
					"newdirectory/nested/nestedfile.txt"
				],
				patterns: [
					{
						from: "nested/**/*",
						context: "directory",
						to: "newdirectory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy a file to a new file", done => {
			runEmit({
				expectedAssetKeys: ["newfile.txt"],
				patterns: [
					{
						from: "file.txt",
						to: "newfile.txt"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should copy a file to a new file with context", done => {
			runEmit({
				expectedAssetKeys: ["newfile.txt"],
				patterns: [
					{
						from: "directoryfile.txt",
						context: "directory",
						to: "newfile.txt"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should multiple files to a new file", done => {
			runEmit({
				expectedAssetKeys: ["newfile.txt", "newbinextension.bin"],
				patterns: [
					{
						from: "file.txt",
						to: "newfile.txt"
					},
					{
						from: "binextension.bin",
						to: "newbinextension.bin"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it('should copy multiple files with same "from"', done => {
			runEmit({
				expectedAssetKeys: ["first/file.txt", "second/file.txt"],
				patterns: [
					{
						from: "file.txt",
						to: "first/file.txt"
					},
					{
						from: "file.txt",
						to: "second/file.txt"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should works with multiple patterns as String", done => {
			runEmit({
				expectedAssetKeys: ["binextension.bin", "file.txt", "noextension"],
				patterns: ["binextension.bin", "file.txt", "noextension"]
			})
				.then(done)
				.catch(done);
		});

		it("should works with multiple patterns as Object", done => {
			runEmit({
				expectedAssetKeys: ["binextension.bin", "file.txt", "noextension"],
				patterns: [
					{
						from: "binextension.bin"
					},
					{
						from: "file.txt"
					},
					{
						from: "noextension"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it('should work with linux path segment separation path when "from" is glob', done => {
			runEmit({
				expectedAssetKeys: ["directory/nested/nestedfile.txt"],
				patterns: [
					{
						from: "directory/nested/*"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it.skip("should exclude path with linux path segment separators", done => {
			runEmit({
				expectedAssetKeys: [
					"[(){}[]!+@escaped-test^$]/hello.txt",
					"[special$directory]/(special-*file).txt",
					"[special$directory]/directoryfile.txt",
					"[special$directory]/nested/nestedfile.txt",
					"dir (86)/file.txt",
					"dir (86)/nesteddir/deepnesteddir/deepnesteddir.txt",
					"dir (86)/nesteddir/nestedfile.txt"
				],
				patterns: [
					{
						from: "!(directory)/**/*.txt"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it.skip('should copy files with "copied" flags', done => {
			expect.assertions(5);

			const expectedAssetKeys = [
				".dottedfile",
				"directoryfile.txt",
				"nested/deep-nested/deepnested.txt",
				"nested/nestedfile.txt"
			];

			run({
				preCopy: {
					additionalAssets: [
						{ name: "foo-bar.txt", data: "Content", info: { custom: true } },
						{
							name: "nested/nestedfile.txt",
							data: "Content",
							info: { custom: true }
						}
					]
				},
				expectedAssetKeys,
				patterns: [
					{
						from: "directory",
						force: true
					}
				]
			})
				.then(({ stats }) => {
					for (const name of expectedAssetKeys) {
						const info = stats.compilation.assetsInfo.get(name);

						expect(info.copied).toBe(true);

						if (name === "nested/nestedfile.txt") {
							expect(info.custom).toBe(true);
						}
					}
				})
				.then(done)
				.catch(done);
		});

		it.skip('should copy files with "copied" flags', done => {
			expect.assertions(5);

			const expectedAssetKeys = [
				"directoryfile.5d7817ed5bc246756d73.txt",
				".dottedfile.5e294e270db6734a42f0",
				"nested/nestedfile.31d6cfe0d16ae931b73c.txt",
				"nested/deep-nested/deepnested.31d6cfe0d16ae931b73c.txt"
			];

			run({
				preCopy: {
					additionalAssets: [
						{
							name: "directoryfile.5d7817ed5bc246756d73.txt",
							data: "Content",
							info: { custom: true }
						}
					]
				},
				expectedAssetKeys,
				patterns: [
					{
						from: "directory",
						to: "[path][name].[contenthash][ext]",
						force: true
					}
				]
			})
				.then(({ stats }) => {
					for (const name of expectedAssetKeys) {
						const info = stats.compilation.assetsInfo.get(name);

						expect(info.immutable).toBe(true);

						if (name === "directoryfile.5d7817ed5bc246756d73.txt") {
							expect(info.immutable).toBe(true);
						}
					}
				})
				.then(done)
				.catch(done);
		});

		it.skip('should copy files and print "copied" in the string representation ', done => {
			expect.assertions(1);

			const expectedAssetKeys = [
				".dottedfile",
				"directoryfile.txt",
				"nested/deep-nested/deepnested.txt",
				"nested/nestedfile.txt"
			];

			run({
				withExistingAsset: true,
				expectedAssetKeys,
				patterns: [
					{
						from: "directory"
					}
				]
			})
				.then(({ stats }) => {
					const stringStats = stats.toString();

					expect(stringStats.match(/\[copied]/g).length).toBe(4);
				})
				.then(done)
				.catch(done);
		});

		it("should work with multi compiler mode", async () => {
			const compiler = rspack([
				{
					mode: "development",
					context: path.resolve(__dirname, "./fixtures"),
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									from: path.resolve(__dirname, "./fixtures/directory")
								}
							]
						})
					],
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/multi-compiler/dist/a")
					}
				},
				{
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									context: path.resolve(__dirname, "./fixtures"),
									from: path.resolve(__dirname, "./fixtures/directory")
								}
							]
						})
					],
					mode: "development",
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/multi-compiler/dist/b")
					}
				}
			]);

			// TODO: output fs system
			// compiler.compilers.forEach((item) => {
			//   // eslint-disable-next-line no-param-reassign
			//   item.outputFileSystem = createFsFromVolume(new Volume());
			// });

			const { stats } = await compile(compiler);

			stats.stats.forEach((item, index) => {
				expect(item.compilation.errors).toMatchSnapshot("errors");
				expect(item.compilation.warnings).toMatchSnapshot("warnings");
				expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
					"assets"
				);
			});
		});

		it("should work with transform fn", async () => {
			const compiler = rspack([
				{
					mode: "development",
					context: path.resolve(__dirname, "./fixtures"),
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									from: path.resolve(__dirname, "./fixtures/directory"),
									transform: source => {
										return source + "transform aaaa";
									}
								}
							]
						})
					],
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/dist/b")
					}
				}
			]);

			const { stats } = await compile(compiler);

			stats.stats.forEach((item, index) => {
				expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
					"assets"
				);
			});
		});

		it("should work with transform async fn", async () => {
			const compiler = rspack([
				{
					mode: "development",
					context: path.resolve(__dirname, "./fixtures"),
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									from: path.resolve(__dirname, "./fixtures/directory"),
									transform: source => {
										expect(Buffer.isBuffer(source)).toBeTruthy();
										return Promise.resolve(source + "transform aaaa");
									}
								}
							]
						})
					],
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/dist/b")
					}
				}
			]);

			const { stats } = await compile(compiler);

			stats.stats.forEach((item, index) => {
				expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
					"assets"
				);
			});
		});

		it("should work with to fn", async () => {
			const compiler = rspack([
				{
					mode: "development",
					context: path.resolve(__dirname, "./fixtures"),
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									from: path.resolve(__dirname, "./fixtures/directory"),
									to: () => {
										return 'directory';
									}
								}
							]
						})
					],
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/dist/b")
					}
				}
			]);

			const { stats } = await compile(compiler);

			stats.stats.forEach((item, index) => {
				expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
					"assets"
				);
			});
		});
	});

	describe("watch mode", () => {
		it('should add the file to the watch list when "from" is a file', done => {
			const expectedAssetKeys = ["file.txt"];

			run({
				patterns: [
					{
						from: "file.txt"
					}
				]
			})
				.then(({ compiler, stats }) => {
					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
				})
				.then(done)
				.catch(done);
		});

		it('should add a directory to the watch list when "from" is a directory', done => {
			run({
				patterns: [
					{
						from: "directory"
					}
				]
			})
				.then(({ stats }) => {
					const { contextDependencies } = stats.compilation;
					const isIncludeDependency = contextDependencies.has(
						path.join(FIXTURES_DIR, "directory")
					);

					expect(isIncludeDependency).toBe(true);
				})
				.then(done)
				.catch(done);
		});

		it('should add a directory to the watch list when "from" is a glob', done => {
			run({
				patterns: [
					{
						from: "directory/**/*"
					}
				]
			})
				.then(({ stats }) => {
					const { contextDependencies } = stats.compilation;
					const isIncludeDependency = contextDependencies.has(
						path.join(FIXTURES_DIR, "directory")
					);

					expect(isIncludeDependency).toBe(true);
				})
				.then(done)
				.catch(done);
		});

		it("should not add the directory to the watch list when glob is a file", done => {
			const expectedAssetKeys = ["directoryfile.txt"];

			run({
				patterns: [
					{
						from: "directory/directoryfile.txt"
					}
				]
			})
				.then(({ compiler, stats }) => {
					expect(Array.from(Object.keys(readAssets(compiler, stats)))).toEqual(
						expectedAssetKeys
					);
				})
				.then(done)
				.catch(done);
		});

		it("should include files that have changed when `from` is a file", done => {
			runChange({
				expectedAssetKeys: ["tempfile1.txt", "tempfile2.txt"],
				newFileLoc1: path.join(FIXTURES_DIR, "watch", "_t5", "tempfile1.txt"),
				newFileLoc2: path.join(FIXTURES_DIR, "watch", "_t5", "tempfile2.txt"),
				patterns: [
					{
						from: "tempfile1.txt",
						context: "watch/_t5"
					},
					{
						from: "tempfile2.txt",
						context: "watch/_t5"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should include all files when `from` is a directory", done => {
			runChange({
				expectedAssetKeys: [".gitkeep", "tempfile1.txt", "tempfile2.txt"],
				newFileLoc1: path.join(
					FIXTURES_DIR,
					"watch",
					"_t4",
					"directory",
					"tempfile1.txt"
				),
				newFileLoc2: path.join(
					FIXTURES_DIR,
					"watch",
					"_t4",
					"directory",
					"tempfile2.txt"
				),
				patterns: [
					{
						from: "watch/_t4/directory"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should include all files when `from` is a glob", done => {
			runChange({
				expectedAssetKeys: [
					"_t3/dest1/tempfile1.txt",
					"_t3/dest1/tempfile2.txt"
				],
				newFileLoc1: path.join(
					FIXTURES_DIR,
					"watch",
					"_t3",
					"directory",
					"tempfile1.txt"
				),
				newFileLoc2: path.join(
					FIXTURES_DIR,
					"watch",
					"_t3",
					"directory",
					"tempfile2.txt"
				),
				patterns: [
					{
						context: "watch/_t3/directory",
						from: "**/*.txt",
						to: "_t3/dest1"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should include all files when multiple patterns used", done => {
			runChange({
				expectedAssetKeys: [
					"_t2/dest1/tempfile1.txt",
					"_t2/dest1/tempfile2.txt",
					"_t2/dest2/tempfile1.txt",
					"_t2/dest2/tempfile2.txt"
				],
				newFileLoc1: path.join(
					FIXTURES_DIR,
					"watch",
					"_t2",
					"directory",
					"tempfile1.txt"
				),
				newFileLoc2: path.join(
					FIXTURES_DIR,
					"watch",
					"_t2",
					"directory",
					"tempfile2.txt"
				),
				patterns: [
					{
						context: "watch/_t2/directory",
						from: "**/*.txt",
						to: "_t2/dest1"
					},
					{
						context: "watch/_t2/directory",
						from: "**/*.txt",
						to: "_t2/dest2"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should include all files when multiple patterns with difference contexts", done => {
			runChange({
				expectedAssetKeys: [
					"_t1/dest1/tempfile1.txt",
					"_t1/dest2/directory/tempfile1.txt",
					"_t1/dest2/tempfile2.txt"
				],
				newFileLoc1: path.join(
					FIXTURES_DIR,
					"watch",
					"_t1",
					"directory",
					"tempfile1.txt"
				),
				newFileLoc2: path.join(FIXTURES_DIR, "watch", "_t1", "tempfile2.txt"),
				patterns: [
					{
						context: "watch/_t1/directory",
						from: "**/*.txt",
						to: "_t1/dest1"
					},
					{
						context: "watch/_t1",
						from: "**/*.txt",
						to: "_t1/dest2"
					}
				]
			})
				.then(done)
				.catch(done);
		});

		it("should run once on child compilation", done => {
			const expectedAssetKeys = ["file.txt"];

			run({
				patterns: [
					{
						from: "file.txt"
					}
				]
			})
				.then(({ compiler, stats }) => {
					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
				})
				.then(done)
				.catch(done);
		});
	});

	describe.skip("cache", () => {
		it('should work with the "memory" cache', async () => {
			const compiler = getCompiler({
				cache: {
					type: "memory"
				},
				plugins: [
					new rspack.CopyRspackPlugin({
						patterns: [
							{
								from: path.resolve(__dirname, "./fixtures/directory")
							}
						]
					})
				]
			});

			const { stats } = await compile(compiler);

			expect(stats.compilation.emittedAssets.size).toBe(5);
			expect(readAssets(compiler, stats)).toMatchSnapshot("assets");
			expect(stats.toJson().errors).toMatchSnapshot("errors");
			expect(stats.toJson().warnings).toMatchSnapshot("warnings");

			await new Promise(async resolve => {
				const { stats: newStats } = await compile(compiler);

				expect(newStats.compilation.emittedAssets.size).toBe(0);
				expect(readAssets(compiler, newStats)).toMatchSnapshot("assets");
				expect(newstats.toJson().errors).toMatchSnapshot("errors");
				expect(newstats.toJson().warnings).toMatchSnapshot("warnings");

				resolve();
			});
		});

		it('should work with the "filesystem" cache', async () => {
			const cacheDirectory = path.resolve(__dirname, "./outputs/.cache/simple");

			try {
				fs.rmdirSync(cacheDirectory, { recursive: true });
			} catch (_) {
				// Nothing
			}

			const compiler = getCompiler({
				cache: {
					type: "filesystem",
					cacheDirectory
				},
				plugins: [
					new rspack.CopyRspackPlugin({
						patterns: [
							{
								from: path.resolve(__dirname, "./fixtures/directory")
							}
						]
					})
				]
			});

			const { stats } = await compile(compiler);

			expect(stats.compilation.emittedAssets.size).toBe(5);
			expect(readAssets(compiler, stats)).toMatchSnapshot("assets");
			expect(stats.toJson().errors).toMatchSnapshot("errors");
			expect(stats.toJson().warnings).toMatchSnapshot("warnings");

			await new Promise(async resolve => {
				const { stats: newStats } = await compile(compiler);

				expect(newStats.compilation.emittedAssets.size).toBe(0);
				expect(readAssets(compiler, newStats)).toMatchSnapshot("assets");
				expect(newstats.toJson().errors).toMatchSnapshot("errors");
				expect(newstats.toJson().warnings).toMatchSnapshot("warnings");

				resolve();
			});
		});

		it('should work with the "filesystem" cache and multi compiler mode', async () => {
			const cacheDirectoryA = path.resolve(
				__dirname,
				"./outputs/.cache/multi-compiler/a"
			);
			const cacheDirectoryB = path.resolve(
				__dirname,
				"./outputs/.cache/multi-compiler/b"
			);

			try {
				fs.rmdirSync(cacheDirectoryA, { recursive: true });
				fs.rmdirSync(cacheDirectoryB, { recursive: true });
			} catch (_) {
				// Nothing
			}

			const compiler = rspack([
				{
					mode: "development",
					context: path.resolve(__dirname, "./fixtures"),
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/multi-compiler/dist/a")
					},
					cache: {
						type: "filesystem",
						cacheDirectory: cacheDirectoryA
					},
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									from: path.resolve(__dirname, "./fixtures/directory")
								}
							]
						})
					]
				},
				{
					mode: "development",
					entry: path.resolve(__dirname, "./helpers/enter.js"),
					output: {
						path: path.resolve(__dirname, "./outputs/multi-compiler/dist/b")
					},
					cache: {
						type: "filesystem",
						cacheDirectory: cacheDirectoryB
					},
					plugins: [
						new rspack.CopyRspackPlugin({
							patterns: [
								{
									context: path.resolve(__dirname, "./fixtures"),
									from: path.resolve(__dirname, "./fixtures/directory")
								}
							]
						})
					]
				}
			]);

			// TODO output fs system
			// compiler.compilers.forEach((item) => {
			//   // eslint-disable-next-line no-param-reassign
			//   item.outputFileSystem = createFsFromVolume(new Volume());
			// });

			const { stats } = await compile(compiler);

			stats.stats.forEach((item, index) => {
				expect(item.compilation.emittedAssets.size).toBe(5);
				expect(item.compilation.errors).toMatchSnapshot("errors");
				expect(item.compilation.warnings).toMatchSnapshot("warnings");
				expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
					"assets"
				);
			});

			await new Promise(async resolve => {
				const { stats: newStats } = await compile(compiler);

				newStats.stats.forEach((item, index) => {
					expect(item.compilation.emittedAssets.size).toBe(0);
					expect(item.compilation.errors).toMatchSnapshot("errors");
					expect(item.compilation.warnings).toMatchSnapshot("warnings");
					expect(readAssets(compiler.compilers[index], item)).toMatchSnapshot(
						"assets"
					);
				});

				resolve();
			});
		});
	});

	describe("stats", () => {
		it("should minify", async () => {
			const compiler = getCompiler({
				mode: "production",
				entry: path.resolve(__dirname, "./helpers/enter-with-asset-modules.js"),
				plugins: [
					new rspack.CopyRspackPlugin({
						patterns: [
							{
								from: path.resolve(__dirname, "./fixtures/js"),
								info: {
									minimized: false
								}
							}
						]
					})
				]
			});

			const { stats } = await compile(compiler);

			expect(readAssets(compiler, stats)).toMatchSnapshot("assets");
		});

		it("should not minify", async () => {
			const compiler = getCompiler({
				mode: "production",
				entry: path.resolve(__dirname, "./helpers/enter-with-asset-modules.js"),
				plugins: [
					new rspack.CopyRspackPlugin({
						patterns: [
							{
								from: path.resolve(__dirname, "./fixtures/js"),
								info: {
									minimized: true
								}
							}
						]
					})
				]
			});

			const { stats } = await compile(compiler);

			expect(readAssets(compiler, stats)).toMatchSnapshot("assets");
		});
	});

	describe.skip("logging", () => {
		it('should logging when "from" is a file', done => {
			const expectedAssetKeys = ["file.txt"];

			run({
				patterns: [
					{
						from: "file.txt"
					}
				]
			})
				.then(({ compiler, stats }) => {
					const root = path.resolve(__dirname).replace(/\\/g, "/");
					const logs = stats.compilation.logging
						.get("copy-rspack-plugin")
						.map(entry =>
							entry.args[0].replace(/\\/g, "/").split(root).join(".")
						)
						.sort();

					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
					expect({ logs }).toMatchSnapshot("logs");
				})
				.then(done)
				.catch(done);
		});

		it('should logging when "from" is a directory', done => {
			const expectedAssetKeys = [
				".dottedfile",
				"directoryfile.txt",
				"nested/deep-nested/deepnested.txt",
				"nested/nestedfile.txt"
			];

			run({
				patterns: [
					{
						from: "directory"
					}
				]
			})
				.then(({ compiler, stats }) => {
					const root = path.resolve(__dirname).replace(/\\/g, "/");
					const logs = stats.compilation.logging
						.get("copy-rspack-plugin")
						.map(entry =>
							entry.args[0].replace(/\\/g, "/").split(root).join(".")
						)
						.sort();

					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
					expect({ logs }).toMatchSnapshot("logs");
				})
				.then(done)
				.catch(done);
		});

		it('should logging when "from" is a glob', done => {
			const expectedAssetKeys = [
				"directory/directoryfile.txt",
				"directory/nested/deep-nested/deepnested.txt",
				"directory/nested/nestedfile.txt"
			];

			run({
				patterns: [
					{
						from: "directory/**",
						globOptions: {
							onlyFiles: false
						}
					}
				]
			})
				.then(({ compiler, stats }) => {
					const root = path.resolve(__dirname).replace(/\\/g, "/");
					const logs = stats.compilation.logging
						.get("copy-rspack-plugin")
						.map(entry =>
							entry.args[0].replace(/\\/g, "/").split(root).join(".")
						)
						.sort();

					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
					expect({ logs }).toMatchSnapshot("logs");
				})
				.then(done)
				.catch(done);
		});

		it("should logging when 'to' is a function", done => {
			const expectedAssetKeys = ["newFile.txt"];

			run({
				patterns: [
					{
						from: "file.txt",
						to() {
							return "newFile.txt";
						}
					}
				]
			})
				.then(({ compiler, stats }) => {
					expect(
						Array.from(Object.keys(readAssets(compiler, stats))).sort()
					).toEqual(expectedAssetKeys);
					expect({ logs }).toMatchSnapshot("logs");
				})
				.then(done)
				.catch(done);
		});
	});
});
