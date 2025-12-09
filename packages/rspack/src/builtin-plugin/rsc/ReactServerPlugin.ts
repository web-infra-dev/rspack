import binding from "@rspack/binding";

import { createBuiltinPlugin, RspackBuiltinPlugin } from "../base";
import rspack, { Compiler } from "../..";
import { Coordinator } from "./coordinator";

export class ReactServerPlugin extends RspackBuiltinPlugin {
	name = "ReactServerPlugin";
	logger: any; // TODO: 类型信息
	coordinator: Coordinator;

	constructor(coordinator: Coordinator) {
		super();
		this.coordinator = coordinator;
	}

	#resolve(serverCompiler: Compiler) {
		this.coordinator.applyServerCompiler(serverCompiler);
		// if (!this.clientCompilerOptions.output) {
		// 	this.clientCompilerOptions.output = {};
		// }
		// if (!this.clientCompilerOptions.output.path) {
		// 	this.clientCompilerOptions.output.path = path.join(
		// 		serverCompiler.context,
		// 		serverCompiler.outputPath,
		// 		"dist/client"
		// 	);
		// }
		// if (!this.clientCompilerOptions.plugins) {
		// 	this.clientCompilerOptions.plugins = [];
		// }
		// this.clientCompilerOptions.plugins.push(
		// 	new ReactClientPlugin(serverCompiler)
		// );

		// const clientCompiler = createCompiler(this.clientCompilerOptions);

		// if (serverCompiler.watchMode) {
		// }

		return this.coordinator.getBinding();
	}

	applyHMRPluginIfAbsent(compiler: Compiler) {
		const HMRPluginExists = compiler.options.plugins.find(
			plugin =>
				plugin && plugin.constructor === rspack.HotModuleReplacementPlugin
		);

		if (HMRPluginExists) {
			this.logger.warn(
				'"hot: true" automatically applies HMR plugin, you don\'t have to add it manually to your Rspack configuration.'
			);
		} else {
			// Apply the HMR plugin
			const plugin = new rspack.HotModuleReplacementPlugin();
			plugin.apply(compiler);
		}
	}

	raw(compiler: Compiler): binding.BuiltinPlugin {
		this.logger = compiler.getInfrastructureLogger("RSCPlugin");

		// new rspack.EntryPlugin(compiler.context, getClientHotEntry(), {
		// 	name: undefined
		// }).apply(compiler);

		// this.applyHMRPluginIfAbsent(compiler);

		const bindingOptions = this.#resolve(compiler);
		return createBuiltinPlugin(this.name, bindingOptions);
	}
}
