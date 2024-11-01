import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import type { Incremental } from "../config";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class ModuleConcatenationPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ModuleConcatenationPlugin;
	affectedHooks = "compilation" as const;

	raw(compiler: Compiler): BuiltinPlugin {
		const incremental = compiler.options.experiments.incremental as Incremental;
		const logger = compiler.getInfrastructureLogger(
			"rspack.ModuleConcatenationPlugin"
		);
		if (incremental.modulesHashes) {
			incremental.modulesHashes = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesHashes` as module concatenation is a global effect. `incremental.modulesHashes` has been overridden to false."
			);
		}
		if (incremental.modulesCodegen) {
			incremental.modulesCodegen = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesCodegen` as module concatenation is a global effect. `incremental.modulesCodegen` has been overridden to false."
			);
		}
		if (incremental.modulesRuntimeRequirements) {
			incremental.modulesRuntimeRequirements = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesRuntimeRequirements` as module concatenation is a global effect. `incremental.modulesRuntimeRequirements` has been overridden to false."
			);
		}
		return createBuiltinPlugin(this.name, undefined);
	}
}
