import { BuiltinPluginName } from "@rspack/binding";

import { Compiler } from "../Compiler";
import { runLoaders } from "../loader-runner";
import { create } from "./base";

export const JsLoaderRspackPlugin = create(
	BuiltinPluginName.JsLoaderRspackPlugin,
	(compiler: Compiler) => runLoaders.bind(null, compiler),
	/* Not Inheretable */
	"thisCompilation"
);
