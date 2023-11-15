import { TestCompilerManager } from "./compiler";
import {
	ITestCompilerManager,
	ECompilerType,
	ITestContext,
	ITesterConfig,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "./type";
import path from "path";

const DEFAULT_COMPILER_NAME = "__default__";

export class TestContext implements ITestContext {
	errors: Error[] = [];
	private compilers: Map<string, ITestCompilerManager<ECompilerType>> =
		new Map();

	constructor(private config: ITesterConfig) {}

	getSource(sub?: string) {
		if (sub) {
			return path.resolve(this.config.src, sub);
		}
		return this.config.src;
	}

	getDist(sub?: string) {
		if (sub) {
			return path.resolve(this.config.dist, sub);
		}
		return this.config.dist;
	}

	async build<T extends ECompilerType>(
		fn: (compiler: TCompiler<T>) => Promise<void>,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		await compiler.build(this, fn);
	}
	options<T extends ECompilerType>(
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T>,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.options(this, fn);
	}
	compiler<T extends ECompilerType>(
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | null,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.compiler(this, fn);
	}
	stats<T extends ECompilerType>(
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | null,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.stats(this, fn);
	}
	result<T extends ECompilerType>(
		fn: <R>(compiler: TCompiler<T> | null, result: R) => R,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.result(this, fn);
	}
	emitError(err: Error | string) {
		this.errors.push(typeof err === "string" ? new Error(err) : err);
	}
	hasError() {
		return !!this.errors.length;
	}
	private getCompilerManage<T extends ECompilerType>(name: string) {
		if (!this.compilers.has(name)) {
			this.compilers.set(name, new TestCompilerManager<T>());
		}
		return this.compilers.get(name) as ITestCompilerManager<T>;
	}
}
