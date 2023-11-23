import { Compiler } from "../Compiler";
import { BuiltinPluginName, create } from "../builtin-plugin/base";

const ModuleFederationRuntimePlugin2 = create(
	BuiltinPluginName.ModuleFederationRuntimePlugin,
	() => undefined
);

export class ModuleFederationRuntimePlugin {
	apply(compiler: Compiler) {
		// TODO: a hack to make sure this runtime is added after ContainerReferencePlugin
		// remove afterPlugin once we make rust side runtime_requirements_in_tree "tapable"
		compiler.hooks.afterPlugins.tap("ModuleFederationRuntimePlugin", () => {
			new ModuleFederationRuntimePlugin2().apply(compiler);
		});
	}
}
