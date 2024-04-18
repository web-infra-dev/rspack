import { ECompilerType } from "../../type";
import {
	IBasicModuleScope,
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "../type";
import { StatsCompilation } from "@rspack/core";
import { WebRunner } from "./web";

interface IHotRunnerOptionsr<T extends ECompilerType = ECompilerType.Rspack>
	extends IBasicRunnerOptions<T> {
	next: (
		callback: (error: Error | null, stats?: StatsCompilation) => void
	) => void;
}

export class HotRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends WebRunner<T> {
	constructor(protected _options: IHotRunnerOptionsr<T>) {
		super(_options);
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: any,
		file: TBasicRunnerFile
	): IBasicModuleScope {
		const moduleScope = super.createModuleScope(requireFn, m, file);
		moduleScope["NEXT"] = this._options.next;
		return moduleScope;
	}
}
