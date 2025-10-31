import { BuiltinPluginName } from "@rspack/binding";
import rspack from "..";
import type { Compiler } from "../Compiler";
import type { RspackOptionsNormalized } from "../config";
import type { Logger } from "../logging/Logger";
import { RemoveDuplicateModulesPlugin } from "./RemoveDuplicateModulesPlugin";

function applyLimits(options: RspackOptionsNormalized, logger: Logger) {
	// concatenateModules is not supported in ESM library mode, it has its own scope hoist algorithm
	options.optimization.concatenateModules = false;

	// chunk rendering is handled by EsmLibraryPlugin
	options.output.chunkFormat = false;

	if (options.output.chunkLoading && options.output.chunkLoading !== "import") {
		logger.warn(
			`\`output.chunkLoading\` should be \`"import"\` or \`false\`, but got ${options.output.chunkLoading}, changed it to \`"import"\``
		);
		options.output.chunkLoading = "import";
	}

	if (options.output.chunkLoading === undefined) {
		options.output.chunkLoading = "import";
	}

	if (options.output.library) {
		options.output.library = undefined;
	}

	const { splitChunks } = options.optimization;

	if (splitChunks) {
		splitChunks.chunks = "all";
		splitChunks.minSize = 0;
		splitChunks.maxAsyncRequests = Infinity;
		splitChunks.maxInitialRequests = Infinity;
		if (splitChunks.cacheGroups) {
			splitChunks.cacheGroups.default = false;
			splitChunks.cacheGroups.defaultVendors = false;
		}
	}
}

export class EsmLibraryPlugin {
	static PLUGIN_NAME = "EsmLibraryPlugin";
	options?: { preserveModules?: string };

	constructor(options?: { preserveModules?: string }) {
		this.options = options;
	}

	apply(compiler: Compiler) {
		const logger = compiler.getInfrastructureLogger(
			EsmLibraryPlugin.PLUGIN_NAME
		);
		applyLimits(compiler.options, logger);
		new RemoveDuplicateModulesPlugin().apply(compiler);

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
