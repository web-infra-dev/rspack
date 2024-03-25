import { BuiltinPluginName } from "@rspack/binding";
import { create } from "./base";
import { runLoaders } from "../loader-runner";
import { Compiler } from "../Compiler";

export const JsLoaderRspackPlugin = create(
	BuiltinPluginName.JsLoaderRspackPlugin,
	(compiler: Compiler) => runLoaders.bind(null, compiler),
	"compilation"
);
