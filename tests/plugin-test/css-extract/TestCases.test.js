/**
 * @jest-environment node
 */

const fs = require("fs");
const path = require("path");
const webpack = require("@rspack/core");
const yn = require("./helpers/yn");

const { CssExtractRspackPlugin } = webpack;
const UPDATE_TEST = global.updateSnapshot;

function clearDirectory(dirPath) {
	let files;

	try {
		files = fs.readdirSync(dirPath);
	} catch (e) {
		return;
	}
	if (files.length > 0) {
		for (let i = 0; i < files.length; i++) {
			const filePath = `${dirPath}/${files[i]}`;

			if (fs.statSync(filePath).isFile()) {
				fs.unlinkSync(filePath);
			} else {
				clearDirectory(filePath);
			}
		}
	}

	fs.rmdirSync(dirPath);
}

const hashRE = /(.*)\.\$.*\$\.(.*)/;

function normalizePath(str) {
	return str.replace(/\\/g, "_").replace(/\\\\/g, "_").replace(/\./g, "_");
}

function getHashes(webpackStats) {
	const statsJson = webpackStats.toJson({
		assets: true
	});
	const fullhash = webpackStats.hash;
	const result = statsJson.assets || [];
	for (const child of statsJson.children || []) {
		result.push(...(child.assets || []));
	}
	return result.reduce((acc, i) => {
		let name = i.name;
		if (i.type === "asset" && i.info?.fullhash?.length) {
			// TODO: handle moduleHashes of asset modules
			if (i.info?.sourceFilename) {
				name = `__ASSET_${normalizePath(i.info.sourceFilename)}_HASH__`;
			} else if (i.info?.fullhash?.length) {
				for (const hash of i.info.fullhash) {
					name = `__ASSET_FULL_${normalizePath(name.replace(`$${hash}$`, "DH").replace(hash, "H"))}_HASH__`;
				}
			}

			acc[name] = i.info.fullhash.join("");
		}
		if (i.info?.contenthash?.length) {
			for (const hash of i.info?.contenthash) {
				name = `__CONTENT_${normalizePath(name.replace(`$${hash}$`, "DH").replace(hash, "H"))}_HASH__`;
			}
			acc[name] = i.info.contenthash.join("");
		}
		return acc;
	}, {
		__FULL_HASH__: fullhash
	})
}

function compareDirectory(actual, expected, webpackStats) {
	const files = fs.readdirSync(expected);
	const hashes = getHashes(webpackStats);
	function recoveryHash(str) {
		for (const [placeholder, hash] of Object.entries(hashes)) {
			str = str.replace(placeholder, hash);
		}
		return str;
	}

	function removeHash(str) {
		for (const [placeholder, hash] of Object.entries(hashes)) {
			str = str.replace(hash, placeholder);
		}
		return str;
	}

	try {
		for (const file of files) {
			const absoluteFilePath = path.resolve(expected, file);

			const stats = fs.lstatSync(absoluteFilePath);

			if (stats.isDirectory()) {
				compareDirectory(
					path.resolve(actual, file),
					path.resolve(expected, file),
					webpackStats
				);
			} else if (stats.isFile()) {
				const expectedPath = path.resolve(expected, file);
				const content = fs.readFileSync(expectedPath, "utf8");
				const actualPath = recoveryHash(path.resolve(actual, file));
				const dir = fs.readdirSync(actual);

				let actualContent;

				if (/^MISSING/.test(content)) {
					expect(fs.existsSync(actualPath)).toBe(false);
				} else {
					try {
						actualContent = removeHash(fs.readFileSync(actualPath, "utf8"));
					} catch (error) {
						if (!UPDATE_TEST) {
							// eslint-disable-next-line no-console
							console.log(`Expected not exist ${expectedPath}`);

							// eslint-disable-next-line no-console
							console.log({ [actual]: dir });

							throw error;
						}
					}

					if (UPDATE_TEST) {
						let match = file.match(hashRE);
						if (match) {
							const [, part1, part2] = match;
							const realFileRE = new RegExp(`${part1}\\.\\$.*\\$\\.${part2}`);
							const realFile = dir.find(value => realFileRE.test(value));

							// update expected
							const newExpectedPath = removeHash(path.resolve(expected, realFile));
							fs.writeFileSync(
								newExpectedPath,
								fs.readFileSync(path.resolve(actual, realFile))
							);
							if (newExpectedPath !== expectedPath) {
								fs.unlinkSync(expectedPath);
							}
						} else {
							fs.writeFileSync(expectedPath, actualContent);
						}
					} else {
						expect(actualContent.replace(/\r\n/g, "\n").trim()).toEqual(content.replace(/\r\n/g, "\n").trim());
					}
				}
			}
		}
	} catch (e) {
		console.log(e);
		throw e;
	}
}

describe("TestCases", () => {
	const casesDirectory = path.resolve(__dirname, "cases");
	const outputDirectory = path.resolve(__dirname, "js");
	const tests = fs.readdirSync(casesDirectory).filter(test => {
		const testDirectory = path.join(casesDirectory, test);
		const filterPath = path.join(testDirectory, "test.filter.js");

		// eslint-disable-next-line global-require, import/no-dynamic-require
		if (fs.existsSync(filterPath) && !require(filterPath)()) {
			describe.skip(test, () => {
				it("filtered", () => { });
			});

			return false;
		}

		return true;
	});

	clearDirectory(outputDirectory);

	for (const directory of tests) {
		if (!/^(\.|_)/.test(directory)) {
			// eslint-disable-next-line no-loop-func
			it(`${directory} should compile to the expected result`, done => {
				if (directory === "serializingBigStrings") {
					clearDirectory(path.resolve(__dirname, "../node_modules/.cache"));
				}

				const directoryForCase = path.resolve(casesDirectory, directory);
				const outputDirectoryForCase = path.resolve(outputDirectory, directory);
				// eslint-disable-next-line import/no-dynamic-require, global-require
				const config = require(path.resolve(
					directoryForCase,
					"webpack.config.js"
				));
				const webpackConfig = Array.isArray(config)
					? config.map(config => {
						return {
							...config,
							optimization: { moduleIds: "named", chunkIds: "named", ...config.optimization },
							output: {
								hashFunction: "xxhash64",
								hashDigestLength: 16,
								...config.output,
							},
							experiments: {
								css: false,
								rspackFuture: {
									bundlerInfo: {
										force: false
									}
								},
								...config.experiments
							}
						};
					})
					: {
						...config,
						optimization: {
							moduleIds: "named",
							chunkIds: "named",
							...config.optimization,
						},
						output: {
							hashFunction: "xxhash64",
							hashDigestLength: 16,
							...config.output,
						},
						experiments: {
							css: false,
							rspackFuture: {
								bundlerInfo: {
									force: false
								}
							},
							...config.experiments
						}
					};
				const { context } = webpackConfig;

				for (const config of [].concat(webpackConfig)) {
					Object.assign(
						config,
						{
							mode: config.mode || "none",
							context: directoryForCase
						},
						config,
						{
							output: Object.assign(
								{
									path: outputDirectoryForCase
								},
								config.output
							),
							plugins:
								config.plugins &&
								config.plugins.map(p => {
									if (p.constructor === CssExtractRspackPlugin) {
										const { options } = p;

										const useImportModule = yn(process.env.OLD_API);

										if (useImportModule === true) {
											options.experimentalUseImportModule = false;
										}
									}

									return p;
								})
						},
						context ? { context } : {}
					);
				}

				webpack(webpackConfig, (error, stats) => {
					if (error) {
						done(error);

						return;
					}

					if (stats.hasErrors()) {
						const errorsPath = path.join(directoryForCase, "./errors.test.js");

						if (fs.existsSync(errorsPath)) {
							const { errors } = stats.compilation;
							// eslint-disable-next-line global-require, import/no-dynamic-require
							const errorFilters = require(errorsPath);
							const filteredErrors = errors.filter(
								// eslint-disable-next-line no-shadow
								error =>
									!errorFilters.some(errorFilter => errorFilter.test(error))
							);

							if (filteredErrors.length > 0) {
								done(new Error(`Errors:\n${filteredErrors.join(",\n")}`));

								return;
							}

							done();

							return;
						}

						done(new Error(stats.toString()));

						return;
					}

					if (stats.hasErrors() && stats.hasWarnings()) {
						done(
							new Error(
								stats.toString({
									context: path.resolve(__dirname, ".."),
									errorDetails: true,
									warnings: true
								})
							)
						);

						return;
					}

					const expectedDirectory = path.resolve(directoryForCase, "expected");
					const expectedDirectoryByVersion = path.join(
						expectedDirectory,
						`webpack-${webpack.version[0]}${yn(process.env.OLD_API) ? "" : "-importModule"
						}`
					);

					if (/^hmr/.test(directory)) {
						let res = fs
							.readFileSync(path.resolve(outputDirectoryForCase, "main.js"))
							.toString();

						const date = Date.now().toString().slice(0, 6);
						const dateRegexp = new RegExp(`${date}\\d+`, "gi");

						res = res.replace(dateRegexp, "");

						const matchAll = res.match(
							/__webpack_require__\.h = function \(\) {\n.*return ("[\d\w].*");\n.*};/i
						);
						const replacer = new Array(matchAll[1].length);

						res = res.replace(
							/__webpack_require__\.h = function \(\) {\n.*return ("[\d\w].*");\n.*};/i,
							`__webpack_require__.h = function () { return "${replacer
								.fill("x")
								.join("")}" }`
						);

						fs.writeFileSync(
							path.resolve(outputDirectoryForCase, "main.js"),
							res
						);
					}

					try {
						if (fs.existsSync(expectedDirectoryByVersion)) {
							compareDirectory(
								outputDirectoryForCase,
								expectedDirectoryByVersion,
								stats
							);
						} else if (fs.existsSync(expectedDirectory)) {
							compareDirectory(outputDirectoryForCase, expectedDirectory, stats);
						}

						const warningsFile = path.resolve(directoryForCase, "warnings.js");

						if (fs.existsSync(warningsFile)) {
							const actualWarnings = stats.toString({
								all: false,
								warnings: true
							});
							// eslint-disable-next-line global-require, import/no-dynamic-require
							const expectedWarnings = require(warningsFile);
							expect(
								actualWarnings
									.replace(/(\(from: .*\))?/g, "")
									.replace(/\*\scss\s(.*)?!/g, "* css /path/to/loader.js!")
									.replace(/\*\scss\s(.*)?!/g, "* css /path/to/loader.js!")
									.replace(/│     at .*\n/g, "")
									.trim()
							).toBe(
								expectedWarnings
									.replace(/\*\scss\s(.*)?!/g, "* css /path/to/loader.js!")
									.replace(/│     at .*\n/g, "")
									.trim()
							);
						}

						const testFile = path.resolve(directoryForCase, "test.js");

						if (fs.existsSync(testFile)) {
							const test = require(testFile);
							test(outputDirectoryForCase, stats);
						}

						done();
					} catch (e) {
						done(e);
					}
				});
			});
		}
	}
});
