import { BuiltinPluginName, type Module } from "@rspack/binding";

import { create } from "../base";

export const BuiltinLazyCompilationPlugin = create(
	BuiltinPluginName.LazyCompilationPlugin,
	(
		module: (args: { module: string; path: string }) => {
			active: boolean;
			data: string;
			client: string;
		},
		cacheable: boolean,
		entries: boolean,
		imports: boolean,
		test?: RegExp | ((module: Module) => boolean)
	) => ({ module, cacheable, imports, entries, test }),
	"thisCompilation"
);
