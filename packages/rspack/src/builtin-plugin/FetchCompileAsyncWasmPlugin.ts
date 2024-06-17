import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const FetchCompileAsyncWasnPlugin = create(
	BuiltinPluginName.FetchCompileAsyncWasnPlugin,
	() => {},
	"thisCompilation"
);
