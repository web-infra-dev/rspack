import { Compiler } from "../Compiler";
import { BuiltinPluginName, create } from "../builtin-plugin/base";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";

const ModuleFederationRuntimePlugin2 = create(
	BuiltinPluginName.ModuleFederationRuntimePlugin,
	() => undefined
);

const compilerToPlugins = new WeakMap<Compiler, Set<string>>();

export class ModuleFederationRuntimePlugin {
	apply(compiler: Compiler) {
		// TODO: a hack to make sure this runtime is added after ContainerReferencePlugin
		// remove afterPlugin once we make rust side runtime_requirements_in_tree "tapable"
		compiler.hooks.afterPlugins.tap(
			{ name: ModuleFederationRuntimePlugin.name, stage: 10 },
			() => {
				const plugins = compilerToPlugins.get(compiler);
				if (plugins) {
					// TODO: move to rust side so don't depend on dataUrl?
					const entry = [...plugins]
						.map(p => `import ${JSON.stringify(p)};`)
						.join("\n");
					new EntryPlugin(compiler.context, `data:text/javascript,${entry}`, {
						name: undefined
					}).apply(compiler);
				}
				new ModuleFederationRuntimePlugin2().apply(compiler);
			}
		);
	}

	static addPlugin(compiler: Compiler, plugin: string) {
		let plugins = compilerToPlugins.get(compiler);
		if (!plugins) {
			compilerToPlugins.set(compiler, (plugins = new Set()));
		}
		plugins.add(plugin);
	}
}
