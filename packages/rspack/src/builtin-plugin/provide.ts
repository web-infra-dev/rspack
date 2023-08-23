import { BuiltinPluginKind, create } from "./base";

export type ProvidePluginOptions = Record<string, string | string[]>;
export const ProvidePlugin = create<
	ProvidePluginOptions,
	Record<string, string[]>
>(BuiltinPluginKind.Provide, provide => {
	const entries = Object.entries(provide).map(([key, value]) => {
		if (typeof value === "string") {
			value = [value];
		}
		return [key, value];
	});
	return Object.fromEntries(entries);
});
