import * as binding from "@rspack/binding";
import { Chunk } from "../Chunk";
import type { Compiler } from "../Compiler";
import { JavascriptModulesPlugin } from "../builtin-plugin";
import { createHash } from "../util/createHash";

type JavaScriptModulesRegisterJsTapKeys =
	`registerJavascriptModules${string}Taps`;
type JavaScriptModulesRegisterTapKeys<T> =
	T extends keyof binding.RegisterJsTaps
		? T extends JavaScriptModulesRegisterJsTapKeys
			? T
			: never
		: never;
type JavaScriptModulesTaps = {
	[K in JavaScriptModulesRegisterTapKeys<
		keyof binding.RegisterJsTaps
	>]: binding.RegisterJsTaps[K];
};

export function createJavaScriptModulesHooksRegisters(
	getCompiler: () => Compiler,
	createTap: Compiler["__internal__create_hook_register_taps"],
	_createMapTap: Compiler["__internal__create_hook_map_register_taps"]
): JavaScriptModulesTaps {
	return {
		registerJavascriptModulesChunkHashTaps: createTap(
			binding.RegisterJsTapKind.JavascriptModulesChunkHash,

			function () {
				return JavascriptModulesPlugin.getCompilationHooks(
					getCompiler().__internal__get_compilation()!
				).chunkHash;
			},

			function (queried) {
				return function (chunk: binding.JsChunk) {
					if (!getCompiler().options.output.hashFunction) {
						throw new Error("'output.hashFunction' cannot be undefined");
					}
					const hash = createHash(getCompiler().options.output.hashFunction!);
					queried.call(Chunk.__from_binding(chunk), hash);
					const digestResult = hash.digest(
						getCompiler().options.output.hashDigest
					);
					return Buffer.from(digestResult);
				};
			}
		)
	};
}
