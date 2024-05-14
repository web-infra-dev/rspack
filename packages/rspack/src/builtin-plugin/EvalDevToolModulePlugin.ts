import {
	BuiltinPluginName,
	RawEvalDevToolModulePluginOptions
} from "@rspack/binding";

import { create } from "./base";

export type { RawEvalDevToolModulePluginOptions as EvalDevToolModulePluginOptions };

export const EvalDevToolModulePlugin = create(
	BuiltinPluginName.EvalDevToolModulePlugin,
	(options: RawEvalDevToolModulePluginOptions) => options,
	"compilation"
);
