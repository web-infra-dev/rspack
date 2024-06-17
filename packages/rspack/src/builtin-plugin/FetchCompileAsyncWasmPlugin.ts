import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const FetchCompileAsyncWasmPlugin = create(
	BuiltinPluginName.FetchCompileAsyncWasmPlugin,
	() => {},
	"thisCompilation"
);
