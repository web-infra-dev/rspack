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
		// TODO: consider introduce `Mutation::Uncacheable` to better handle this
		if (incremental.moduleIds) {
			incremental.moduleIds = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.moduleIds` as module concatenation is a global effect. `incremental.moduleIds` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesHashes) {
			incremental.modulesHashes = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesHashes` as module concatenation is a global effect. `incremental.modulesHashes` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesCodegen) {
			incremental.modulesCodegen = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesCodegen` as module concatenation is a global effect. `incremental.modulesCodegen` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.modulesRuntimeRequirements) {
			incremental.modulesRuntimeRequirements = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.modulesRuntimeRequirements` as module concatenation is a global effect. `incremental.modulesRuntimeRequirements` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.chunksRuntimeRequirements) {
			incremental.chunksRuntimeRequirements = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.chunksRuntimeRequirements` as module concatenation is a global effect. `incremental.chunksRuntimeRequirements` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.chunksHashes) {
			incremental.chunksHashes = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.chunksHashes` as module concatenation is a global effect. `incremental.chunksHashes` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		if (incremental.chunksRender) {
			incremental.chunksRender = false;
			logger.warn(
				"`optimization.concatenateModules` can't be used with `incremental.chunksRender` as module concatenation is a global effect. `incremental.chunksRender` has been overridden to false. We recommend enabling incremental only in the development mode."
			);
		}
		return createBuiltinPlugin(this.name, undefined);
	}
}
