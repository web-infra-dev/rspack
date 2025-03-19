import path from "node:path";

import type { ECompilerType } from "../../type";
import type {
	IBasicModuleScope,
	TBasicRunnerFile,
	TRunnerRequirer
} from "../type";
import type { IBasicRunnerOptions } from "./basic";
import { FakeDocumentWebRunner } from "./web/fake";

interface IWatchRunnerOptions<T extends ECompilerType = ECompilerType.Rspack>
	extends IBasicRunnerOptions<T> {
	stepName: string;
	isWeb: boolean;
	state: Record<string, any>;
}

export class WatchRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends FakeDocumentWebRunner<T> {
	constructor(protected _watchOptions: IWatchRunnerOptions<T>) {
		super(_watchOptions);
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: any,
		file: TBasicRunnerFile
	): IBasicModuleScope {
		const moduleScope = super.createModuleScope(requireFn, m, file);
		moduleScope.__dirname = path.dirname(file.path);
		moduleScope.document = this.globalContext!.document;
		moduleScope.STATE = this._watchOptions.state;
		moduleScope.WATCH_STEP = this._watchOptions.stepName;
		moduleScope.__STATS__ = this._options.stats;
		return moduleScope;
	}

	run(file: string) {
		return super.run(file);
	}
}
