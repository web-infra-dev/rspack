import { BuiltinPluginKind, create } from "./base";

export const SwcCssMinimizerPlugin = create(
	BuiltinPluginKind.SwcCssMinimizer,
	(options?: any /* TODO: extend more options */) => undefined
);
