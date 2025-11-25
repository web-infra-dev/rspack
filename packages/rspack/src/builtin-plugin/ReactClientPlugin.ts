import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";
import { Compiler } from "..";
import { GET_COMPILER_ID } from "../Compiler";

export const ReactClientPlugin = create(
	BuiltinPluginName.ReactClientPlugin,
	function (serverCompiler: Compiler) {
		return {
			getServerCompilerId() {
				return serverCompiler[GET_COMPILER_ID]();
			}
		};
	}
);
