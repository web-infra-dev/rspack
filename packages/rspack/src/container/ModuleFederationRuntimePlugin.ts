import { Compiler } from "../Compiler";
import { BuiltinPluginName, create } from "../builtin-plugin/base";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";

const ModuleFederationRuntimePlugin2 = create(
	BuiltinPluginName.ModuleFederationRuntimePlugin,
	() => undefined
);

export class ModuleFederationRuntimePlugin {
	plugins: string[] = [];

	apply(compiler: Compiler) {
		// TODO: a hack to make sure this runtime is added after ContainerReferencePlugin
		// remove afterPlugin once we make rust side runtime_requirements_in_tree "tapable"
		compiler.hooks.afterPlugins.tap(
			{ name: ModuleFederationRuntimePlugin.name, stage: 10 },
			() => {
				// TODO: move to rust side so don't depend on dataUrl
				const entry = this.plugins.map(p => `import "${p}";`).join("\n");
				new EntryPlugin(compiler.context, `data:text/javascript,${entry}`, {
					name: undefined
				}).apply(compiler);
				new ModuleFederationRuntimePlugin2().apply(compiler);
			}
		);
	}

	addPlugin(dep: string) {
		this.plugins.push(dep);
	}
}
