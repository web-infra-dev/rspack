import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import type { Incremental } from "../config";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class NaturalModuleIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.NaturalModuleIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(compiler: Compiler): BuiltinPlugin {
		const incremental = compiler.options.experiments.incremental as Incremental;
		const logger = compiler.getInfrastructureLogger(
			"rspack.NaturalModuleIdsPlugin"
		);
		if (incremental.moduleIds) {
			incremental.moduleIds = false;
			logger.warn(
				"`optimization.moduleIds = 'natural'` can't be used with `incremental.moduleIds` as natural module ids is a global effect. `incremental.moduleIds` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		return createBuiltinPlugin(this.name, undefined);
	}
}
