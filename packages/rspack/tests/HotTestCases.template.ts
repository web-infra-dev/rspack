import path from "path";
import fs from "graceful-fs";
import vm from "vm";
import rimraf from "rimraf";
import checkArrayExpectation from "./checkArrayExpectation";
import createLazyTestEnv from "./helpers/createLazyTestEnv";
import { Compiler, rspack, Stats } from "@rspack/core";

export function describeCases(config: {
	name: string;
	target: string;
	casesPath: string;
	hot: boolean;
	incrementalRebuild?: boolean;
}) {
	const casesPath = path.join(__dirname, config.casesPath);
	const categories = fs
		.readdirSync(casesPath)
		.filter(dir => fs.statSync(path.join(casesPath, dir)).isDirectory())
		.map(cat => ({
			name: cat,
			tests: fs
				.readdirSync(path.join(casesPath, cat))
				.filter(folder => folder.indexOf("_") < 0)
		}));
	describe(config.name, () => {
		categories.forEach(category => {
			describe(category.name, () => {
				category.tests.forEach(testName => {
					const testDirectory = path.join(casesPath, category.name, testName);
					const filterPath = path.join(testDirectory, "test.filter.js");
					if (fs.existsSync(filterPath) && !require(filterPath)(config)) {
						describe.skip(testName, () => {
							it("filtered", () => {});
						});
						return;
					}

					describe(testName, () => {
						let compiler: undefined | Compiler;

						afterAll(callback => {
							if (!compiler) {
								return;
							}
							compiler.close(callback);
							// compiler = undefined;
						});

						it(
							testName + " should compile",
							done => {
								const outputDirectory = path.join(
									__dirname,
									"js",
									`hot-cases-${config.name}`,
									category.name,
									testName
								);
								rimraf.sync(outputDirectory);
								// TODO: should remove it.
								const changedFiles = path.join(
									testDirectory,
									"changed-file.js"
								);

								const fakeUpdateLoaderOptions = {
									updateIndex: 0
								};
								const configPath = path.join(
									testDirectory,
									"webpack.config.js"
								);
								const options = getOptions(
									configPath,
									testDirectory,
									outputDirectory,
									fakeUpdateLoaderOptions,
									config.target,
									config.hot,
									config.incrementalRebuild
								);
								compiler = rspack(options);
								compiler.build(err => {
									if (err) {
										return done(err);
									}

									const stats = new Stats(compiler!.compilation);
									const jsonStats = stats.toJson();
									if (
										checkArrayExpectation(
											testDirectory,
											jsonStats,
											"error",
											"Error",
											done
										)
									) {
										return;
									}
									if (
										checkArrayExpectation(
											testDirectory,
											jsonStats,
											"warning",
											"Warning",
											done
										)
									) {
										return;
									}
									const urlToPath = (url: string) => {
										if (url.startsWith("https://test.cases/path/")) {
											url = url.slice(24);
										}
										return path.resolve(outputDirectory, `./${url}`);
									};
									const urlToRelativePath = (url: string) => {
										if (url.startsWith("https://test.cases/path/")) {
											url = url.slice(24);
										}
										return `./${url}`;
									};
									const window = {
										fetch: async (url: string) => {
											try {
												const buffer: any = await new Promise(
													(resolve, reject) => {
														fs.readFile(urlToPath(url), (err, b) => {
															err ? reject(err) : resolve(b);
														});
													}
												);
												return {
													status: 200,
													ok: true,
													json: async () => JSON.parse(buffer.toString("utf-8"))
												};
											} catch (err) {
												if (err.code === "ENOENT") {
													return {
														status: 404,
														ok: false
													};
												}
												throw err;
											}
										},
										importScripts: (url: string) => {
											expect(url).toMatch(/^https:\/\/test\.cases\/path\//);
											_require(urlToRelativePath(url));
										},
										document: {
											createElement(type) {
												return {
													_type: type,
													_attrs: {},
													setAttribute(name, value) {
														this._attrs[name] = value;
													},
													getAttribute(name) {
														return this._attrs[name];
													},
													removeAttribute(name) {
														delete this._attrs[name];
													},
													parentNode: {
														removeChild(node) {
															// ok
														}
													},
													// css link
													sheet: {
														disabled: false
													}
												};
											},
											head: {
												children: [],
												insertBefore(element: any, before: any) {
													element.parentNode = this;
													this.children.unshift(element);
													Promise.resolve().then(() => {
														if (element.onload) {
															element.onload({ type: "load", target: element });
														}
													});
												},
												appendChild(element: any) {
													element.parentNode = this;
													this.children.push(element);
													if (element._type === "script") {
														Promise.resolve().then(() => {
															_require(urlToRelativePath(element.src));
															if (element.onload) {
																element.onload({
																	type: "load",
																	target: element
																});
															}
														});
													} else {
														if (element.onload) {
															element.onload({ type: "load", target: element });
														}
													}
												},
												removeChild(node) {
													const index = this.children.indexOf(node);
													this.children.splice(index, 1);
												}
											},
											getElementsByTagName(name: string) {
												if (name === "head") {
													return [this.head];
												}
												if (name === "script" || name === "link") {
													return this.head.children.filter(
														i => i._type === name
													);
												}
												throw Error("No supported");
											}
										},
										Worker: require("./helpers/createFakeWorker")({
											outputDirectory
										}),
										EventSource: require("./helpers/EventSourceForNode"),
										location: {
											href: "https://test.cases/path/index.html",
											origin: "https://test.cases",
											toString() {
												return "https://test.cases/path/index.html";
											}
										}
									};

									function _next(callback) {
										fakeUpdateLoaderOptions.updateIndex++;
										if (!compiler) {
											throw Error("can't find compiler");
										}
										// should delete after removed `rebuild`
										let changed = [];
										try {
											changed = require(changedFiles);
										} catch (err) {}
										if (changed.length === 0) {
											throw Error("can not found changed files");
										}
										compiler.rebuild(new Set(changed), new Set(), err => {
											if (err) {
												return callback(err);
											}
											const jsonStats = new Stats(
												compiler!.compilation
											).toJson();
											if (
												checkArrayExpectation(
													testDirectory,
													jsonStats,
													"error",
													"errors" + fakeUpdateLoaderOptions.updateIndex,
													"Error",
													callback
												)
											) {
												return;
											}
											if (
												checkArrayExpectation(
													testDirectory,
													jsonStats,
													"warning",
													"warnings" + fakeUpdateLoaderOptions.updateIndex,
													"Warning",
													callback
												)
											) {
												return;
											}
											callback(null, jsonStats);
										});
									}
									function _require(module: string) {
										if (module.startsWith("./")) {
											const p = path.join(outputDirectory, module);
											if (module.endsWith(".json")) {
												return JSON.parse(fs.readFileSync(p, "utf8"));
											} else {
												const code =
													"(function(require, module, exports, __dirname, __filename, it, beforeEach, afterEach, expect, jest, self, window, fetch, document, importScripts, Worker, EventSource, NEXT, STATS) {" +
													"global.expect = expect;" +
													'function nsObj(m) { Object.defineProperty(m, Symbol.toStringTag, { value: "Module" }); return m; }\n' +
													fs.readFileSync(p, "utf-8") +
													"\n})";
												const fn = vm.runInThisContext(code, p);
												const m = {
													exports: {}
												};
												fn.call(
													m.exports,
													_require,
													m,
													m.exports,
													outputDirectory,
													p,
													_it,
													_beforeEach,
													_afterEach,
													expect,
													jest,
													window,
													window,
													window.fetch,
													window.document,
													window.importScripts,
													window.Worker,
													window.EventSource,
													_next,
													jsonStats
												);
												return m.exports;
											}
										} else {
											return require(module);
										}
									}
									let promise = Promise.resolve();
									const info = stats.toJson({});
									// here it is diffrent from webpack, because we add hotCases/chunk/multi-chunk-single-runtime
									if (
										config.target === "web" ||
										config.target === "webworker"
									) {
										for (const file of info.entrypoints!.main.assets) {
											if (file.name.endsWith(".js")) {
												_require(`./${file.name}`);
											} else {
												const cssElement =
													window.document.createElement("link");
												// @ts-expect-error
												cssElement.href = file.name;
												// @ts-expect-error
												cssElement.rel = "stylesheet";
												window.document.head.appendChild(cssElement);
											}
										}
									} else {
										const assets = info.entrypoints!.main.assets.filter(s =>
											s.name.endsWith(".js")
										);
										const result = _require(
											`./${assets[assets.length - 1].name}`
										);
										if (typeof result === "object" && "then" in result)
											promise = promise.then(() => result);
									}
									promise.then(
										() => {
											if (getNumberOfTests() < 1) {
												return done(
													new Error("No tests exported by test case")
												);
											}
											done();
										},
										error => {
											console.log(err);
											done(error);
										}
									);
								});
							},
							20000
						);

						const {
							it: _it,
							beforeEach: _beforeEach,
							afterEach: _afterEach,
							getNumberOfTests
						} = createLazyTestEnv(20000);
					});
				});
			});
		});
	});
}

function getOptions(
	configPath: string,
	testDirectory: string,
	outputDirectory: string,
	fakeUpdateLoaderOptions: any,
	target: string,
	hot: boolean,
	incrementalRebuild?: boolean
): Record<string, string> {
	let options: any = {};
	if (fs.existsSync(configPath)) {
		Object.assign(options, require(configPath));
	}
	if (!options.mode) {
		options.mode = "development";
	}
	if (!options.devtool) {
		options.devtool = false;
	}
	if (!options.context) {
		options.context = testDirectory;
	}
	if (!options.entry) {
		options.entry = "./index.js";
	}
	if (!options.output) {
		options.output = {};
	}
	if (!options.output.path) {
		options.output.path = outputDirectory;
	}
	if (!options.output.filename) {
		options.output.filename == "bundle.js";
	}
	if (!options.output.chunkFilename) {
	}
	if (options.output.publicPath === undefined) {
		options.output.publicPath = "https://test.cases/path/";
	}
	if (!options.module) {
		options.module = {};
	}
	if (!options.module.rules) {
		options.module.rules = [];
	}
	options.module.rules.push({
		test: /\.(js|css|json)/,
		use: [
			{
				loader: path.join(__dirname, "hotCases", "fake-update-loader.js"),
				options: fakeUpdateLoaderOptions
			}
		]
	});

	if (!options.target) {
		options.target = target;
	}
	if (!options.plugins) {
		options.plugins = [];
	}
	if (typeof incrementalRebuild === "boolean") {
		options.experiments = {
			incrementalRebuild
		};
	}

	options.devServer = {
		...options.devServer,
		hot
	};
	return options;
}
