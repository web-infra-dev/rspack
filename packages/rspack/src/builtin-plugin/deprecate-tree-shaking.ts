import { BuiltinPluginKind, create } from "./base";

export const TreeShakingPlugin = create<
	{ enable: boolean | "module"; production: boolean },
	string
>(BuiltinPluginKind.TreeShaking, ({ enable, production }) => {
	return enable !== undefined
		? enable.toString()
		: production
		? "true"
		: "false";
});
