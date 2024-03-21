import { readConfigFile } from "../helper";
import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicTaskProcessor } from "./basic";
import { merge } from "webpack-merge";

export interface IMultiTaskProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	defaultOptions?: (
		index: number,
		context: ITestContext
	) => TCompilerOptions<T>;
	overrideOptions?: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void;
	findBundle?: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	compilerType: ECompilerType.Rspack;
	name: string;
	configFiles?: string[];
	runable: boolean;
}

export class MultiTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	extends BasicTaskProcessor<T>
	implements ITestProcessor
{
	protected multiCompilerOptions: TCompilerOptions<T>[] = [];
	constructor(protected _multiOptions: IMultiTaskProcessorOptions<T>) {
		super({
			runable: _multiOptions.runable,
			compilerType: _multiOptions.compilerType as T,
			findBundle: (context, _) => {
				if (typeof _multiOptions.findBundle !== "function") {
					return [];
				}
				return this.multiCompilerOptions.reduce<string[]>(
					(res, compilerOptions, index) => {
						const curBundles = _multiOptions.findBundle!(
							index,
							context,
							compilerOptions
						);

						const bundles = Array.isArray(curBundles)
							? curBundles
							: curBundles
								? [curBundles]
								: [];

						const multiFileIndexMap: Record<string, number[]> =
							context.getValue(_multiOptions.name, "multiFileIndexMap") || {};
						for (const bundle of bundles) {
							multiFileIndexMap[bundle] = [
								...(multiFileIndexMap[bundle] || []),
								index
							];
						}
						context.setValue(
							_multiOptions.name,
							"multiFileIndexMap",
							multiFileIndexMap
						);
						return [
							...res,
							...(Array.isArray(bundles) ? bundles : bundles ? [bundles] : [])
						];
					},
					[]
				);
			},
			name: _multiOptions.name
		});
	}

	async config(context: ITestContext) {
		this.multiCompilerOptions = [];
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this._multiOptions.configFiles
		)
			? readConfigFile(
					this._multiOptions.configFiles!.map(i => context.getSource(i))
				)
			: [{}];

		for (let [index, options] of caseOptions.entries()) {
			const compilerOptions = merge(
				typeof this._multiOptions.defaultOptions === "function"
					? this._multiOptions.defaultOptions!(index, context)
					: {},
				options
			);

			if (typeof this._multiOptions.overrideOptions === "function") {
				this._multiOptions.overrideOptions!(index, context, compilerOptions);
			}

			this.multiCompilerOptions.push(compilerOptions);
		}

		const compiler = this.getCompiler(context);
		compiler.setOptions(this.multiCompilerOptions as any);
		context.setValue(
			this._options.name,
			"multiCompilerOptions",
			this.multiCompilerOptions
		);
	}
}
