import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const SwcCssMinimizerRspackPlugin = create(
	BuiltinPluginName.SwcCssMinimizerRspackPlugin,
	(options?: any /* TODO: extend more options */) => undefined
);
