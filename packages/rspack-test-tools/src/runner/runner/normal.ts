import { URL } from "url";

import { ECompilerType } from "../../type";
import { IBasicModuleScope, TBasicRunnerFile, TRunnerRequirer } from "../type";
import { EsmRunner } from "./esm";

export class NormalRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends EsmRunner<T> {
	protected createBaseModuleScope(): IBasicModuleScope {
		const baseModuleScope = Object.assign(super.createBaseModuleScope(), {
			process,
			global,
			URL,
			Buffer,
			setImmediate
		});
		return baseModuleScope;
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: { exports: unknown },
		file: TBasicRunnerFile
	): IBasicModuleScope {
		const moduleScope = super.createModuleScope(requireFn, m, file);
		delete moduleScope.define;
		return moduleScope;
	}
}
