import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import type { Incremental } from "../config";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class MangleExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.MangleExportsPlugin;
	affectedHooks = "compilation" as const;

	constructor(private deterministic: boolean) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const incremental = compiler.options.experiments.incremental as Incremental;
		const logger = compiler.getInfrastructureLogger(
			"rspack.MangleExportsPlugin"
		);
		if (incremental.modulesHashes) {
			incremental.modulesHashes = false;
			logger.warn(
				"`optimization.mangleExports` can't be used with `incremental.modulesHashes` as export mangling is a global effect. `incremental.modulesHashes` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesCodegen) {
			incremental.modulesCodegen = false;
			logger.warn(
				"`optimization.mangleExports` can't be used with `incremental.modulesCodegen` as export mangling is a global effect. `incremental.modulesCodegen` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesRuntimeRequirements) {
			incremental.modulesRuntimeRequirements = false;
			logger.warn(
				"`optimization.mangleExports` can't be used with `incremental.modulesRuntimeRequirements` as export mangling is a global effect. `incremental.modulesRuntimeRequirements` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		return createBuiltinPlugin(this.name, this.deterministic);
	}
}
