import { ECompilerType } from "../type";
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

export class BasicRunner<T extends ECompilerType = ECompilerType.Rspack> {
	protected requirers: Map<string, TRunnerRequirer> = new Map();
	constructor(protected options: IBasicRunnerOptions<T>) {}

	run(file: string) {
		const globalContext = this.createGlobalContext();
		const baseModuleScope = this.createBaseModuleScope();
		if (typeof this.options.testConfig.moduleScope === "function") {
			this.options.testConfig.moduleScope(baseModuleScope);
		}
		this.createRunner(baseModuleScope, globalContext);
		return this.requirers.get("entry")!(
			this.options.dist,
			file.startsWith("./") ? file : `./${file}`
		);
	}

	protected createGlobalContext() {
		return {
			console: console,
			expect: expect,
			setTimeout: setTimeout,
			clearTimeout: clearTimeout
		};
	}

	protected createBaseModuleScope() {
		return {
			console: console,
			it: this.options.env.it,
			beforeEach: this.options.env.beforeEach,
			afterEach: this.options.env.afterEach,
			expect,
			jest,
			__STATS__: this.options.stats,
			nsObj: (m: Object) => {
				Object.defineProperty(m, Symbol.toStringTag, {
					value: "Module"
				});
				return m;
			}
		};
	}

	protected createSubModuleScope(
		requireFn: TRunnerRequirer,
		m: any,
		file: TBasicRunnerFile,
		baseModuleScope: IBasicModuleScope
	): IBasicModuleScope {
		return {
			...baseModuleScope,
			require: requireFn.bind(null, path.dirname(file.path)),
			module: m,
			exports: m.exports,
			__dirname: path.dirname(file.path),
			__filename: file.path,
			_globalAssign: { expect }
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
				subPath: getSubPath(p)
			};
		} else {
			return null;
		}
	}

	protected createMissRequirer(
		moduleScope: IBasicModuleScope,
		globalContext: IBasicGlobalContext
	): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			const modulePathStr = modulePath as string;
			const modules = this.options.testConfig.modules;
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

	protected createCjsRequirer(
		moduleScope: IBasicModuleScope,
		globalContext: IBasicGlobalContext
	): TRunnerRequirer {
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
			const currentModuleScope = this.createSubModuleScope(
				this.requirers.get("entry")!,
				m,
				file,
				moduleScope
			);

			if (this.options.testConfig.moduleScope) {
				this.options.testConfig.moduleScope(currentModuleScope);
			}
			if (!this.options.runInNewContext) {
				file.content = `Object.assign(global, _globalAssign);\n ${file.content}`;
			}
			const args = Object.keys(currentModuleScope);
			const argValues = args.map(arg => currentModuleScope[arg]);
			const code = `(function(${args.join(", ")}) {${file.content}\n})`;
			const fn = this.options.runInNewContext
				? vm.runInNewContext(code, globalContext, file.path)
				: vm.runInThisContext(code, file.path);
			fn.call(
				this.options.testConfig.nonEsmThis
					? this.options.testConfig.nonEsmThis(modulePath)
					: m.exports,
				...argValues
			);
			return m.exports;
		};
	}

	protected createRunner(
		moduleScope: IBasicModuleScope,
		globalContext: IBasicGlobalContext
	) {
		this.requirers.set(
			"miss",
			this.createMissRequirer(moduleScope, globalContext)
		);
		this.requirers.set(
			"entry",
			this.createCjsRequirer(moduleScope, globalContext)
		);
	}
}
