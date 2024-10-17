import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

const map = {
	foo: "dts"
};

export const SplitSharedModulesPlugin = create(
	BuiltinPluginName.SplitSharedModulesPlugin,
	() => {
		return {};
	}
);
