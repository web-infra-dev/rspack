import { RawWebWorkerTemplatePluginOptions } from "@rspack/binding";
import { BuiltinPluginName, create } from "./base";

export type WebWorkerTemplateOptions = {};

export const WebWorkerTemplatePlugin = create(
	BuiltinPluginName.WebWorkerTemplatePlugin,
	(): RawWebWorkerTemplatePluginOptions => {
		return {};
	}
);
