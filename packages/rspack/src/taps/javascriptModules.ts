import * as binding from "@rspack/binding";
import { Chunk } from "../Chunk";
import { JavascriptModulesPlugin } from "../builtin-plugin";
import { toBuffer } from "../util";
import { createHash } from "../util/createHash";
import type { CreatePartialRegisters } from "./types";

export const createJavaScriptModulesHooksRegisters: CreatePartialRegisters<
	`JavascriptModules`
> = (getCompiler, createTap, createMapTap) => {
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
					let digestResult: Buffer | string;
					if (getCompiler().options.output.hashDigest) {
						digestResult = hash.digest(
							getCompiler().options.output.hashDigest as string
						);
					} else {
						digestResult = hash.digest();
					}
					return toBuffer(digestResult);
				};
			}
		)
	};
};
