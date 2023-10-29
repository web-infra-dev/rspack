import { BuiltinPluginName, create } from "./base";

export type WebWorkerTemplateOptions = {};

export const WebWorkerTemplatePlugin = create(
	BuiltinPluginName.WebWorkerTemplatePlugin,
	() => undefined
);
