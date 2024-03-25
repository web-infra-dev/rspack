import { BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";
import { Compiler, LibraryType } from "..";

const enabledTypes = new WeakMap();

const getEnabledTypes = (compiler: Compiler) => {
	let set = enabledTypes.get(compiler);
	if (set === undefined) {
		set = new Set();
		enabledTypes.set(compiler, set);
	}
	return set;
};

export class EnableLibraryPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.EnableLibraryPlugin;

	constructor(private type: LibraryType) {
		super();
	}

	static setEnabled(compiler: Compiler, type: LibraryType) {
		getEnabledTypes(compiler).add(type);
	}

	static checkEnabled(compiler: Compiler, type: LibraryType) {
		if (!getEnabledTypes(compiler).has(type)) {
			throw new Error(
				`Library type "${type}" is not enabled. ` +
					"EnableLibraryPlugin need to be used to enable this type of library. " +
					'This usually happens through the "output.enabledLibraryTypes" option. ' +
					'If you are using a function as entry which sets "library", you need to add all potential library types to "output.enabledLibraryTypes". ' +
					"These types are enabled: " +
					Array.from(getEnabledTypes(compiler)).join(", ")
			);
		}
	}

	raw(compiler: Compiler): BuiltinPlugin | undefined {
		const { type } = this;

		// Only enable once
		const enabled = getEnabledTypes(compiler);
		if (enabled.has(type)) return;
		enabled.add(type);

		return createBuiltinPlugin(this.name, type);
	}
}
