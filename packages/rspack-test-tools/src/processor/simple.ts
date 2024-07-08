import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "../type";

export interface ISimpleProcessorOptions<T extends ECompilerType> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
	compilerType: T;
	name: string;
	build?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	compiler?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (
		context: ITestContext,
		compiler: TCompiler<T>,
		stats: TCompilerStats<T>
	) => Promise<void>;
}

export class SimpleTaskProcessor<T extends ECompilerType>
	implements ITestProcessor
{
	constructor(protected _options: ISimpleProcessorOptions<T>) {}

	async config(context: ITestContext) {
		const compiler = this.getCompiler(context);
		if (typeof this._options.options === "function") {
			compiler.setOptions(this._options.options(context));
		}
	}

	async compiler(context: ITestContext) {
		const compiler = this.getCompiler(context);
		const instance = compiler.createCompiler();
		if (typeof this._options.compiler === "function") {
			await this._options.compiler(context, instance);
		}
	}

	async build(context: ITestContext) {
		const compiler = this.getCompiler(context);
		if (typeof this._options.build === "function") {
			await this._options.build(context, compiler.getCompiler()!);
		} else {
			await compiler.build();
		}
	}

	async run(env: ITestEnv, context: ITestContext) {}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (typeof this._options.check === "function") {
			await this._options.check(context, compiler.getCompiler()!, stats!);
		}
	}

	async before(context: ITestContext): Promise<void> {}
	async after(context: ITestContext): Promise<void> {
		const compiler = this.getCompiler(context);
		await compiler.close();
	}
	async beforeAll(context: ITestContext): Promise<void> {}
	async afterAll(context: ITestContext) {}

	protected getCompiler(context: ITestContext) {
		return context.getCompiler(this._options.name, this._options.compilerType);
	}
}
