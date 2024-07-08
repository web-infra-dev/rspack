import path from "path";

import FakeDocument from "../../helper/legacy/FakeDocument";
import { ECompilerType } from "../../type";
import { IBasicModuleScope, TBasicRunnerFile, TRunnerRequirer } from "../type";
import { IBasicRunnerOptions } from "./basic";
import { CommonJsRunner } from "./cjs";

interface IWatchRunnerOptions<T extends ECompilerType = ECompilerType.Rspack>
	extends IBasicRunnerOptions<T> {
	stepName: string;
}

export class WatchRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends CommonJsRunner<T> {
	private document: any;
	private state: Record<string, any> = {};
	constructor(protected _watchOptions: IWatchRunnerOptions<T>) {
		super(_watchOptions);
		this.document = new FakeDocument(_watchOptions.dist);
	}

	protected createGlobalContext() {
		const globalContext = super.createGlobalContext();
		globalContext["document"] = this.document;
		return globalContext;
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: any,
		file: TBasicRunnerFile
	): IBasicModuleScope {
		const moduleScope = super.createModuleScope(requireFn, m, file);
		moduleScope["__dirname"] = path.dirname(file.path);
		moduleScope["document"] = this.globalContext!["document"];
		moduleScope["STATE"] = this.state;
		moduleScope["WATCH_STEP"] = this._watchOptions.stepName;
		return moduleScope;
	}
}
