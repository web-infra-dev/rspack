import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const WebWorkerTemplatePlugin = create(
	BuiltinPluginName.WebWorkerTemplatePlugin,
	() => undefined
);
