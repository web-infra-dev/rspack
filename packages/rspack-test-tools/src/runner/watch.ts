import { ECompilerType } from "../type";
import { BasicRunner } from "./basic";
import {
	IBasicModuleScope,
	IBasicRunnerOptions,
	TBasicRunnerFile,
	TRunnerRequirer
} from "./type";
import fs from "fs";
import path from "path";
import FakeDocument from "../helper/legacy/FakeDocument";

interface IWatchRunnerOptions<T extends ECompilerType = ECompilerType.Rspack>
	extends IBasicRunnerOptions<T> {
	stepName: string;
}

export class WatchRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
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
		moduleScope["STATS_JSON"] = moduleScope.__STATS__.toJson({
			errorDetails: true
		} as any);
		moduleScope["STATE"] = this.state;
		moduleScope["WATCH_STEP"] = this._watchOptions.stepName;
		return moduleScope;
	}
}
