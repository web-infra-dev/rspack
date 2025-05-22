import {
	BuiltinPluginName,
	type RawRstestPluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type RstestPluginArgument =
	| Partial<Omit<RawRstestPluginOptions, "handler">>
	| ((percentage: number, msg: string, ...args: string[]) => void)
	| undefined;

export const RstestPlugin = create(
	BuiltinPluginName.RstestPlugin,
	(rstest: RawRstestPluginOptions): RawRstestPluginOptions => {
		return rstest;
	}
);
