import { readConfigFile } from "../helper";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompiler,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { BasicTaskProcessor } from "./basic";

export interface IMultiTaskProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	preOptions?: (index: number, context: ITestContext) => TCompilerOptions<T>;
	postOptions?: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void;
	getCompiler: (
		index: number,
		context: ITestContext
	) => (options: TCompilerOptions<T>) => TCompiler<T>;
	getBundle: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	testConfig: TTestConfig<T>;
	name: string;
	configFiles?: string[];
}

export class MultiTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestProcessor
{
	private processors: Map<string, BasicTaskProcessor<T>> = new Map();
	constructor(protected options: IMultiTaskProcessorOptions<T>) {}

	async config(context: ITestContext) {
		const source = context.getSource();
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this.options.configFiles
		)
			? readConfigFile(source, this.options.configFiles!)
			: [{}];

		for (let [index, options] of caseOptions.entries()) {
			const taskId = `${this.options.name}[${index + 1}]`;
			const processor = new BasicTaskProcessor<T>({
				preOptions:
					typeof this.options.preOptions === "function"
						? context => this.options.preOptions!(index, context)
						: () => ({}),
				postOptions:
					typeof this.options.postOptions === "function"
						? (context, options) =>
								this.options.postOptions!(index, context, options)
						: () => {},
				getBundle: (context, options) =>
					this.options.getBundle(index, context, options),
				getCompiler: context => this.options.getCompiler(index, context),
				getCompilerOptions: () => options,
				testConfig: {
					...this.options.testConfig,
					beforeExecute: undefined,
					afterExecute: undefined
				},
				name: taskId
			});
			await processor.config(context);
			this.processors.set(taskId, processor);
		}
	}

	async compiler(context: ITestContext) {
		for (const processor of this.processors.values()) {
			await processor.compiler(context);
		}
	}

	async build(context: ITestContext) {
		for (const processor of this.processors.values()) {
			await processor.build(context);
		}
	}

	async run(env: ITestEnv, context: ITestContext) {
		if (typeof this.options.testConfig.beforeExecute === "function") {
			this.options.testConfig.beforeExecute();
		}
		for (const processor of this.processors.values()) {
			await processor.run(env, context);
		}
		if (typeof this.options.testConfig.afterExecute === "function") {
			this.options.testConfig.afterExecute();
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		for (const processor of this.processors.values()) {
			await processor.check(env, context);
		}
	}
}
