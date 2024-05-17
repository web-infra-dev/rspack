import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";

export const APIPlugin = create(BuiltinPluginName.APIPlugin, () => {});
