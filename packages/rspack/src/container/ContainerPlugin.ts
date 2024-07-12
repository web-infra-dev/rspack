import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawContainerPluginOptions
} from "@rspack/binding";

import type { Compiler } from "../Compiler";
import {
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import {
	type EntryRuntime,
	type FilenameTemplate,
	type LibraryOptions,
	getRawLibrary
} from "../config";
import { parseOptions } from "../container/options";
import { ShareRuntimePlugin } from "../sharing/ShareRuntimePlugin";

export type ContainerPluginOptions = {
	exposes: Exposes;
	filename?: FilenameTemplate;
	library?: LibraryOptions;
	name: string;
	runtime?: EntryRuntime;
	shareScope?: string;
	enhanced?: boolean;
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
	_options;

	constructor(options: ContainerPluginOptions) {
		super();
		this._options = {
			name: options.name,
			shareScope: options.shareScope || "default",
			library: options.library || {
				type: "var",
				name: options.name
			},
			runtime: options.runtime,
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
			),
			enhanced: options.enhanced ?? false
		};
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const { name, shareScope, library, runtime, filename, exposes, enhanced } =
			this._options;
		if (!compiler.options.output.enabledLibraryTypes!.includes(library.type)) {
			compiler.options.output.enabledLibraryTypes!.push(library.type);
		}
		new ShareRuntimePlugin(this._options.enhanced).apply(compiler);

		const rawOptions: RawContainerPluginOptions = {
			name,
			shareScope,
			library: getRawLibrary(library),
			runtime,
			filename,
			exposes: exposes.map(([key, r]) => ({ key, ...r })),
			enhanced
		};
		return createBuiltinPlugin(this.name, rawOptions);
	}
}
