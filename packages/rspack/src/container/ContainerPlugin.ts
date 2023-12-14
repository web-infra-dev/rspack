import { RawContainerPluginOptions, BuiltinPlugin } from "@rspack/binding";
import {
	BuiltinPluginName,
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import {
	EntryRuntime,
	Filename,
	LibraryOptions,
	getRawEntryRuntime,
	getRawLibrary
} from "../config";
import { isNil } from "../util";
import { parseOptions } from "../container/options";
import { Compiler } from "../Compiler";
import { ModuleFederationRuntimePlugin } from "./ModuleFederationRuntimePlugin";

export type ContainerPluginOptions = {
	exposes: Exposes;
	filename?: Filename;
	library?: LibraryOptions;
	name: string;
	runtime?: EntryRuntime;
	shareScope?: string;
};
export type Exposes = (ExposesItem | ExposesObject)[] | ExposesObject;
export type ExposesItem = string;
export type ExposesItems = ExposesItem[];
export type ExposesObject = {
	[k: string]: ExposesConfig | ExposesItem | ExposesItems;
};
export type ExposesConfig = {
	import: ExposesItem | ExposesItems;
	name?: string;
};

export class ContainerPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ContainerPlugin;
	_options: RawContainerPluginOptions;
	_library;

	constructor(options: ContainerPluginOptions) {
		super();
		const library = (this._library = options.library || {
			type: "var",
			name: options.name
		});
		const runtime = options.runtime;
		this._options = {
			name: options.name,
			shareScope: options.shareScope || "default",
			library: getRawLibrary(library),
			runtime: !isNil(runtime) ? getRawEntryRuntime(runtime) : undefined,
			filename: options.filename,
			exposes: parseOptions(
				options.exposes,
				item => ({
					import: Array.isArray(item) ? item : [item],
					name: undefined
				}),
				item => ({
					import: Array.isArray(item.import) ? item.import : [item.import],
					name: item.name || undefined
				})
			).map(([key, r]) => ({ key, ...r }))
		};
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const library = this._library;
		if (!compiler.options.output.enabledLibraryTypes!.includes(library.type)) {
			compiler.options.output.enabledLibraryTypes!.push(library.type);
		}
		ModuleFederationRuntimePlugin.addPlugin(
			compiler,
			require.resolve("../sharing/initializeSharing.js")
		);
		return createBuiltinPlugin(this.name, this._options);
	}
}
