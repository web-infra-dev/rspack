import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import type { Incremental } from "../config";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class FlagDependencyUsagePlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.FlagDependencyUsagePlugin;
	affectedHooks = "compilation" as const;

	constructor(private global: boolean) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const incremental = compiler.options.experiments.incremental as Incremental;
		const logger = compiler.getInfrastructureLogger(
			"rspack.FlagDependencyUsagePlugin"
		);
		if (incremental.modulesHashes) {
			incremental.modulesHashes = false;
			logger.warn(
				"`optimization.usedExports` can't be used with `incremental.modulesHashes` as export usage is a global effect. `incremental.modulesHashes` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesCodegen) {
			incremental.modulesCodegen = false;
			logger.warn(
				"`optimization.usedExports` can't be used with `incremental.modulesCodegen` as export usage is a global effect. `incremental.modulesCodegen` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesRuntimeRequirements) {
			incremental.modulesRuntimeRequirements = false;
			logger.warn(
				"`optimization.usedExports` can't be used with `incremental.modulesRuntimeRequirements` as export usage is a global effect. `incremental.modulesRuntimeRequirements` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		return createBuiltinPlugin(this.name, this.global);
	}
}
