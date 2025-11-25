import { BuiltinPluginName } from "@rspack/binding";

import { create } from "./base";
import { createCompiler } from "../rspack";
import { Compiler, ReactClientPlugin, RspackOptions } from "..";
import path from "path";

export const ReactServerComponentsPlugin = create(
	BuiltinPluginName.ReactServerComponentsPlugin,
	function (this: Compiler, clientCompilerOptions: RspackOptions = {}) {
		if (!clientCompilerOptions.output) {
			clientCompilerOptions.output = {};
		}
		if (!clientCompilerOptions.output.path) {
			clientCompilerOptions.output.path = path.join(
				this.context,
				this.outputPath,
				"dist/client"
			);
		}
		if (!clientCompilerOptions.plugins) {
			clientCompilerOptions.plugins = [];
		}
		clientCompilerOptions.plugins.push(new ReactClientPlugin(this));

		const clientCompiler = createCompiler(clientCompilerOptions);

		return {
			compile() {
				console.log("client compiler 开始构建");

				return new Promise((resolve, reject) => {
					clientCompiler.run((error, compilation) => {
						console.log("client compiler 构建完成", error);
						if (error) {
							return reject(error);
						}
						resolve(undefined);
					});
				});
			}
		};
	}
);
