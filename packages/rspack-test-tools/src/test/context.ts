import { TestCompilerManager } from "../compiler";
import {
	ITestCompilerManager,
	ECompilerType,
	ITestContext,
	ITesterConfig,
	TCompiler,
	TCompilerOptions,
	TCompilerStats,
	TTestRunResult
} from "../type";
import path from "path";

const DEFAULT_COMPILER_NAME = "__default__";

export class TestContext implements ITestContext {
	errors: Map<string, Error[]> = new Map();
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
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T> | void,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.options(this, fn);
	}
	compiler<T extends ECompilerType>(
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | void,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.compiler(this, fn);
	}
	stats<T extends ECompilerType>(
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | void,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.stats(this, fn);
	}
	result<T extends ECompilerType>(
		fn: (
			compiler: TCompiler<T> | null,
			result: TTestRunResult
		) => TTestRunResult | void,
		name = DEFAULT_COMPILER_NAME
	) {
		const compiler = this.getCompilerManage<T>(name);
		compiler.result(this, fn);
	}
	emitError(err: Error | string, name = DEFAULT_COMPILER_NAME) {
		const errors = this.errors.get(name) || [];
		errors.push(typeof err === "string" ? new Error(err) : err);
		this.errors.set(name, errors);
	}
	hasError() {
		return !!Array.from(this.errors.values()).reduce(
			(res, arr) => res + arr.length,
			0
		);
	}
	getError(name = DEFAULT_COMPILER_NAME) {
		return this.errors.get(name);
	}
	clearError(name = DEFAULT_COMPILER_NAME) {
		this.errors.delete(name);
	}
	private getCompilerManage<T extends ECompilerType>(name: string) {
		if (!this.compilers.has(name)) {
			this.compilers.set(name, new TestCompilerManager<T>(name));
		}
		return this.compilers.get(name) as ITestCompilerManager<T>;
	}
}
