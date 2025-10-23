import { BuiltinPluginName } from "@rspack/binding";
import rspack from "..";
import type { Compiler } from "../Compiler";
import type { RspackOptionsNormalized } from "../config";
import { RemoveDuplicateModulesPlugin } from "./RemoveDuplicateModulesPlugin";

export class EsmLibraryPlugin {
	static PLUGIN_NAME = "EsmLibraryPlugin";
	options?: { preserveModules?: string };

	constructor(options?: { preserveModules?: string }) {
		this.options = options;
	}

	apply(compiler: Compiler) {
		new RemoveDuplicateModulesPlugin().apply(compiler);

		const { splitChunks } = compiler.options.optimization;

		if (splitChunks) {
			splitChunks.chunks = "all";
			splitChunks.minSize = 0;
		}

		let err;
		if ((err = checkConfig(compiler.options))) {
			throw new rspack.WebpackError(
				`Conflicted config for ${EsmLibraryPlugin.PLUGIN_NAME}: ${err}`
			);
		}

		compiler.__internal__registerBuiltinPlugin({
			name: BuiltinPluginName.EsmLibraryPlugin,
			options: {
				preserveModules: this.options?.preserveModules
			}
		});
	}
}

function checkConfig(config: RspackOptionsNormalized): string | undefined {
	if (config.optimization.concatenateModules) {
		return "You should disable `config.optimization.concatenateModules`";
	}

	if (config.output.chunkFormat !== false) {
		return "You should disable default chunkFormat by `config.output.chunkFormat = false`";
	}
}
