import { BuiltinPluginKind, create } from "./base";

export const NoEmitAssetsPlugin = create<undefined, undefined>(
	BuiltinPluginKind.NoEmitAssets,
	() => undefined
);
