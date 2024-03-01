import { ECompilerType, ITestRunner } from "../type";
import vm from "vm";
import path from "path";
import fs from "fs";
import {
	IBasicGlobalContext,
	IBasicModuleScope,
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "./type";

const define = function (...args: unknown[]) {
	const factory = args.pop() as () => {};
	factory();
};
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

export class BasicRunner<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestRunner
{
	protected globalContext: IBasicGlobalContext | null = null;
	protected baseModuleScope: IBasicModuleScope | null = null;
	protected requirers: Map<string, TRunnerRequirer> = new Map();
	constructor(protected _options: IBasicRunnerOptions<T>) {}

	run(file: string): Promise<unknown> {
		this.globalContext = this.createGlobalContext();
		this.baseModuleScope = this.createBaseModuleScope();
		if (typeof this._options.testConfig.moduleScope === "function") {
			this._options.testConfig.moduleScope(this.baseModuleScope);
		}
		this.createRunner();
		const res = this.getRequire()(
			this._options.dist,
			file.startsWith("./") ? file : `./${file}`
		);
		if (typeof res === "object" && "then" in res) {
			return res;
		} else {
			return Promise.resolve(res);
		}
	}

	getRequire(): TRunnerRequirer {
		return this.requirers.get("entry")!;
	}

	protected createGlobalContext(): IBasicGlobalContext {
		return {
			console: console,
			expect: expect,
			setTimeout: ((
				cb: (...args: any[]) => void,
				ms: number | undefined,
				...args: any
			) => {
				let timeout = setTimeout(cb, ms, ...args);
				timeout.unref();
				return timeout;
			}) as typeof setTimeout,
			clearTimeout: clearTimeout
		};
	}

	protected createBaseModuleScope(): IBasicModuleScope {
		return {
			console: console,
			it: this._options.env.it,
			beforeEach: this._options.env.beforeEach,
			afterEach: this._options.env.afterEach,
			expect,
			jest,
			__STATS__: this._options.stats,
			nsObj: (m: Object) => {
				Object.defineProperty(m, Symbol.toStringTag, {
					value: "Module"
				});
				return m;
			}
		};
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: { exports: unknown },
		file: TBasicRunnerFile
	): IBasicModuleScope {
		return {
			...this.baseModuleScope!,
			require: requireFn.bind(null, path.dirname(file.path)),
			module: m,
			exports: m.exports,
			__dirname: path.dirname(file.path),
			__filename: file.path,
			_globalAssign: { expect },
			define
		};
	}

	protected getFile(
		modulePath: string[] | string,
		currentDirectory: string
	): TBasicRunnerFile | null {
		if (Array.isArray(modulePath)) {
			return {
				path: path.join(currentDirectory, ".array-require.js"),
				content: `module.exports = (${modulePath
					.map(arg => {
						return `require(${JSON.stringify(`./${arg}`)})`;
					})
					.join(", ")});`,
				subPath: ""
			};
		} else if (isRelativePath(modulePath)) {
			const p = path.join(currentDirectory, modulePath);
			return {
				path: p,
				content: fs.readFileSync(p, "utf-8"),
				subPath: getSubPath(modulePath)
			};
		} else {
			return null;
		}
	}

	protected createMissRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			const modulePathStr = modulePath as string;
			const modules = this._options.testConfig.modules;
			if (modules && modulePathStr in modules) {
				return modules[modulePathStr];
			} else {
				return require(
					modulePathStr.startsWith("node:")
						? modulePathStr.slice(5)
						: modulePathStr
				);
			}
		};
	}

	protected createCjsRequirer(): TRunnerRequirer {
		const requireCache = Object.create(null);

		return (currentDirectory, modulePath, context = {}) => {
			let file = context["file"] || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path in requireCache) {
				return requireCache[file.path].exports;
			}

			const m = {
				exports: {}
			};
			requireCache[file.path] = m;
			const currentModuleScope = this.createModuleScope(
				this.getRequire(),
				m,
				file
			);

			if (this._options.testConfig.moduleScope) {
				this._options.testConfig.moduleScope(currentModuleScope);
			}

			if (!this._options.runInNewContext) {
				file.content = `Object.assign(global, _globalAssign);\n ${file.content}`;
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

	protected preExecute(code: string, file: TBasicRunnerFile) {}
	protected postExecute(m: Object, file: TBasicRunnerFile) {}

	protected createRunner() {
		this.requirers.set("miss", this.createMissRequirer());
		this.requirers.set("entry", this.createCjsRequirer());
	}
}
