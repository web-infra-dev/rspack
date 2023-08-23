import { BuiltinPluginKind, create } from "./base";

export const SwcCssMinimizerPlugin = create<{}, undefined>(
	BuiltinPluginKind.SwcCssMinimizer,
	() => undefined
);
