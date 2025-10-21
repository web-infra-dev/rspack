import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import vm, { SourceTextModule } from "node:vm";
import asModule from "../../helper/legacy/asModule";
import createFakeWorker from "../../helper/legacy/createFakeWorker";
import urlToRelativePath from "../../helper/legacy/urlToRelativePath";
import {
	type ECompilerType,
	EEsmMode,
	type IGlobalContext,
	type IModuleScope,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStatsCompilation,
	type TModuleObject,
	type TRunnerFile,
	type TRunnerRequirer,
	type TTestConfig
} from "../../type";

declare global {
	var printLogger: boolean;
}

const isRelativePath = (p: string) => /^\.\.?\//.test(p);
const getSubPath = (p: string) => {
	const lastSlash = p.lastIndexOf("/");
	let firstSlash = p.indexOf("/");

	if (lastSlash !== -1 && firstSlash !== lastSlash) {
		if (firstSlash !== -1) {
			let next = p.indexOf("/", firstSlash + 1);
			let dir = p.slice(firstSlash + 1, next);

			while (dir === ".") {
				firstSlash = next;
				next = p.indexOf("/", firstSlash + 1);
				dir = p.slice(firstSlash + 1, next);
			}
		}

		return p.slice(firstSlash + 1, lastSlash + 1);
	}
	return "";
};

const cached = new Map<string, TRunnerFile>();

export interface INodeRunnerOptions<T extends ECompilerType> {
	env: ITestEnv;
	stats?: () => TCompilerStatsCompilation<T>;
	name: string;
	runInNewContext?: boolean;
	testConfig: TTestConfig<T>;
	source: string;
	dist: string;
	compilerOptions: TCompilerOptions<T>;
	cachable?: boolean;
}
export class NodeRunner<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestRunner
{
	protected requireCache = Object.create(null);

	protected globalContext: IGlobalContext | null = null;
	protected baseModuleScope: IModuleScope | null = null;
	protected requirers: Map<string, TRunnerRequirer> = new Map();
	constructor(protected _options: INodeRunnerOptions<T>) {}

	run(file: string): Promise<unknown> {
		if (!this.globalContext) {
			this.globalContext = this.createGlobalContext();
		}
		this.baseModuleScope = this.createBaseModuleScope();
		if (typeof this._options.testConfig.moduleScope === "function") {
			this._options.testConfig.moduleScope(
				this.baseModuleScope,
				this._options.stats,
				this._options.compilerOptions
			);
		}
		this.createRunner();
		const res = this.getRequire()(
			this._options.dist,
			file.startsWith("./") || file.startsWith("https://test.cases/")
				? file
				: `./${file}`
		);
		if (typeof res === "object" && "then" in res) {
			return res;
		}
		return Promise.resolve(res);
	}

	getRequire(): TRunnerRequirer {
		const entryRequire = this.requirers.get("entry")!;
		return (currentDirectory, modulePath, context = {}) => {
			const p = Array.isArray(modulePath)
				? modulePath
				: modulePath.split("?")[0]!;
			return entryRequire(currentDirectory, p, context);
		};
	}

	getGlobal(name: string): unknown {
		return ((this.globalContext || {}) as Record<string, unknown>)[name];
	}

	protected createGlobalContext(): IGlobalContext {
		return {
			console: {
				log: (...args: any[]) => {
					if (printLogger) {
						console.log(...args);
					}
				},
				warn: (...args: any[]) => {
					if (printLogger) {
						console.warn(...args);
					}
				},
				error: (...args: any[]) => {
					console.error(...args);
				},
				info: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				debug: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				trace: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				assert: (...args: any[]) => {
					console.assert(...args);
				},
				clear: () => {
					console.clear();
				}
			},
			setTimeout: ((
				cb: (...args: any[]) => void,
				ms: number | undefined,
				...args: any
			) => {
				const timeout = setTimeout(cb, ms, ...args);
				timeout.unref();
				return timeout;
			}) as typeof setTimeout,
			clearTimeout: clearTimeout
		};
	}
	protected createBaseModuleScope(): IModuleScope {
		const baseModuleScope: IModuleScope = {
			console: this.globalContext!.console,
			setTimeout: this.globalContext!.setTimeout,
			clearTimeout: this.globalContext!.clearTimeout,
			nsObj: (m: Object) => {
				Object.defineProperty(m, Symbol.toStringTag, {
					value: "Module"
				});
				return m;
			},
			process,
			URL,
			Blob,
			Symbol,
			Buffer,
			setImmediate,
			self: this.globalContext,
			__MODE__: this._options.compilerOptions.mode,
			__SNAPSHOT__: path.join(this._options.source, "__snapshot__"),
			Worker: createFakeWorker(this._options.env, {
				outputDirectory: this._options.dist
			}),
			...this._options.env
		};
		return baseModuleScope;
	}
	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: TModuleObject,
		file: TRunnerFile
	): IModuleScope {
		const requirer: TRunnerRequirer & {
			webpackTestSuiteRequire?: boolean;
		} = requireFn.bind(null, path.dirname(file.path));
		requirer.webpackTestSuiteRequire = true;
		return {
			...this.baseModuleScope!,
			require: requirer,
			module: m,
			exports: m.exports,
			__dirname: path.dirname(file.path),
			__filename: file.path,
			_globalAssign: {
				expect: this._options.env.expect
			}
		};
	}

	protected getFile(
		modulePath: string[] | string,
		currentDirectory: string
	): TRunnerFile | null {
		const cacheKey = `${currentDirectory}|${modulePath}`;
		if (this._options.cachable && cached.has(cacheKey)) {
			return cached.get(cacheKey)!;
		}
		let res = null;
		if (Array.isArray(modulePath)) {
			res = {
				path: path.join(currentDirectory, ".array-require.js"),
				content: `module.exports = (${modulePath
					.map(arg => {
						return `require(${JSON.stringify(`./${arg}`)})`;
					})
					.join(", ")});`,
				subPath: ""
			};
		} else if (modulePath.startsWith("https://test.cases/")) {
			const relativePath = urlToRelativePath(modulePath);
			const absPath = path.join(currentDirectory, relativePath);
			res = {
				path: absPath,
				content: fs.readFileSync(absPath, "utf-8"),
				subPath: ""
			};
		} else if (isRelativePath(modulePath)) {
			const p = path.join(currentDirectory, modulePath);
			res = {
				path: p,
				content: fs.readFileSync(p, "utf-8"),
				subPath: getSubPath(modulePath)
			};
		} else if (path.isAbsolute(modulePath)) {
			res = {
				path: modulePath,
				content: fs.readFileSync(modulePath, "utf-8"),
				subPath: "absolute_path"
			};
		}
		if (this._options.cachable && res) {
			cached.set(cacheKey, res);
		}
		return res;
	}

	protected preExecute(code: string, file: TRunnerFile) {}
	protected postExecute(m: Object, file: TRunnerFile) {}

	protected createRunner() {
		this.requirers.set("cjs", this.createCjsRequirer());
		this.requirers.set("esm", this.createEsmRequirer());
		this.requirers.set("miss", this.createMissRequirer());
		this.requirers.set("json", this.createJsonRequirer());
		this.requirers.set("entry", (currentDirectory, modulePath, context) => {
			const file = this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}
			if (file.path.endsWith(".json")) {
				return this.requirers.get("json")!(currentDirectory, modulePath, {
					...context,
					file
				});
			}
			if (
				file.path.endsWith(".mjs") &&
				this._options.compilerOptions.experiments?.outputModule
			) {
				return this.requirers.get("esm")!(currentDirectory, modulePath, {
					...context,
					file
				});
			}
			return this.requirers.get("cjs")!(currentDirectory, modulePath, {
				...context,
				file
			});
		});
	}

	protected createMissRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			const modulePathStr = modulePath as string;
			const modules = this._options.testConfig.modules;
			if (modules && modulePathStr in modules) {
				return modules[modulePathStr];
			}
			return require(
				modulePathStr.startsWith("node:")
					? modulePathStr.slice(5)
					: modulePathStr
			);
		};
	}

	protected createJsonRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			if (Array.isArray(modulePath)) {
				throw new Error("Array module path is not supported in hot cases");
			}
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}
			return JSON.parse(
				fs.readFileSync(path.join(this._options.dist, modulePath), "utf-8")
			);
		};
	}

	protected createCjsRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			if (modulePath === "@rspack/test-tools") {
				return require("@rspack/test-tools");
			}
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path in this.requireCache) {
				return this.requireCache[file.path].exports;
			}

			const m = {
				exports: {},
				webpackTestSuiteModule: true
			};
			this.requireCache[file.path] = m;

			if (!this._options.runInNewContext) {
				file.content = `Object.assign(global, _globalAssign);\n ${file.content}`;
			}

			const currentModuleScope = this.createModuleScope(
				this.getRequire(),
				m,
				file
			);

			if (this._options.testConfig.moduleScope) {
				this._options.testConfig.moduleScope(
					currentModuleScope,
					this._options.stats,
					this._options.compilerOptions
				);
			}
			if (file.content.includes("__STATS__") && this._options.stats) {
				currentModuleScope.__STATS__ = this._options.stats();
			}
			if (file.content.includes("__STATS_I__")) {
				const statsIndex = this._options.stats?.()?.__index__;
				if (typeof statsIndex === "number") {
					currentModuleScope.__STATS_I__ = statsIndex;
				}
			}
			const args = Object.keys(currentModuleScope);
			const argValues = args.map(arg => currentModuleScope[arg]);
			const code = `(function(${args.join(", ")}) {
        ${file.content}
      })`;

			this.preExecute(code, file);
			const fn = this._options.runInNewContext
				? vm.runInNewContext(code, this.globalContext!, file.path)
				: vm.runInThisContext(code, file.path);
			fn.call(
				this._options.testConfig.nonEsmThis
					? this._options.testConfig.nonEsmThis(modulePath)
					: m.exports,
				...argValues
			);

			this.postExecute(m, file);
			return m.exports;
		};
	}

	protected createEsmRequirer(): TRunnerRequirer {
		const esmContext = vm.createContext(this.baseModuleScope!, {
			name: "context for esm"
		});
		const esmCache = new Map<string, SourceTextModule>();
		const esmIdentifier = this._options.name;

		return (currentDirectory, modulePath, context = {}) => {
			if (!SourceTextModule) {
				throw new Error(
					"Running this test requires '--experimental-vm-modules'.\nRun with 'node --experimental-vm-modules node_modules/jest-cli/bin/jest'."
				);
			}
			const _require = this.getRequire();
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.content.includes("__STATS__")) {
				esmContext.__STATS__ = this._options.stats?.();
			}

			if (file.content.includes("__STATS_I__")) {
				const statsIndex = this._options.stats?.()?.__index__;
				if (typeof statsIndex === "number") {
					esmContext.__STATS_I__ = statsIndex;
				}
			}

			let esm = esmCache.get(file.path);
			if (!esm) {
				esm = new SourceTextModule(file.content, {
					identifier: `${esmIdentifier}-${file.path}`,
					// no attribute
					url: `${pathToFileURL(file.path).href}?${esmIdentifier}`,
					context: esmContext,
					initializeImportMeta: (
						meta: { url: string; dirname?: string; filename?: string },
						_: any
					) => {
						meta.url = pathToFileURL(file!.path).href;
						meta.dirname = path.dirname(file!.path);
						meta.filename = file!.path;
					},
					importModuleDynamically: async (
						specifier: any,
						module: { context: any }
					) => {
						const result = await _require(path.dirname(file!.path), specifier, {
							esmMode: EEsmMode.Evaluated
						});
						return await asModule(result, module.context);
					}
				} as any);
				esmCache.set(file.path, esm);
			}
			if (context.esmMode === EEsmMode.Unlinked) return esm;
			return (async () => {
				if (esm.status === "unlinked") {
					await esm.link(async (specifier, referencingModule) => {
						return await asModule(
							await _require(
								path.dirname(
									referencingModule.identifier
										? referencingModule.identifier.slice(
												esmIdentifier.length + 1
											)
										: fileURLToPath((referencingModule as any).url)
								),
								specifier,
								{
									esmMode: EEsmMode.Unlinked
								}
							),
							referencingModule.context,
							true
						);
					});
				}

				if ((esm as any).instantiate) (esm as any).instantiate();
				await esm.evaluate();
				if (context.esmMode === EEsmMode.Evaluated) {
					return esm;
				}
				const ns = esm.namespace as {
					default: unknown;
				};
				return ns.default && ns.default instanceof Promise ? ns.default : ns;
			})();
		};
	}
}
