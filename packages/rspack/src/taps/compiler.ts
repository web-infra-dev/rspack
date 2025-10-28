import binding from "@rspack/binding";
import type { CreatePartialRegisters } from "./types";

export const createCompilerHooksRegisters: CreatePartialRegisters<
	`Compiler`
> = (getCompiler, createTap) => {
	return {
		registerCompilerThisCompilationTaps: createTap(
			binding.RegisterJsTapKind.CompilerThisCompilation,

			function () {
				return getCompiler().hooks.thisCompilation;
			},

			function (queried) {
				return function (native: binding.JsCompilation) {
					getCompiler().__internal__create_compilation(native);
					return queried.call(
						getCompiler().__internal__get_compilation()!,
						getCompiler().__internal__get_compilation_params()!
					);
				};
			}
		),
		registerCompilerCompilationTaps: createTap(
			binding.RegisterJsTapKind.CompilerCompilation,

			function () {
				return getCompiler().hooks.compilation;
			},

			function (queried) {
				return function () {
					return queried.call(
						getCompiler().__internal__get_compilation()!,
						getCompiler().__internal__get_compilation_params()!
					);
				};
			}
		),
		registerCompilerMakeTaps: createTap(
			binding.RegisterJsTapKind.CompilerMake,

			function () {
				return getCompiler().hooks.make;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!
					);
				};
			}
		),
		registerCompilerFinishMakeTaps: createTap(
			binding.RegisterJsTapKind.CompilerFinishMake,

			function () {
				return getCompiler().hooks.finishMake;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!
					);
				};
			}
		),
		registerCompilerShouldEmitTaps: createTap(
			binding.RegisterJsTapKind.CompilerShouldEmit,

			function () {
				return getCompiler().hooks.shouldEmit;
			},

			function (queried) {
				return function () {
					return queried.call(getCompiler().__internal__get_compilation()!);
				};
			}
		),
		registerCompilerEmitTaps: createTap(
			binding.RegisterJsTapKind.CompilerEmit,

			function () {
				return getCompiler().hooks.emit;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!
					);
				};
			}
		),
		registerCompilerAfterEmitTaps: createTap(
			binding.RegisterJsTapKind.CompilerAfterEmit,

			function () {
				return getCompiler().hooks.afterEmit;
			},

			function (queried) {
				return async function () {
					return await queried.promise(
						getCompiler().__internal__get_compilation()!
					);
				};
			}
		),
		registerCompilerAssetEmittedTaps: createTap(
			binding.RegisterJsTapKind.CompilerAssetEmitted,

			function () {
				return getCompiler().hooks.assetEmitted;
			},

			function (queried) {
				return async function ({
					filename,
					targetPath,
					outputPath
				}: binding.JsAssetEmittedArgs) {
					return queried.promise(filename, {
						compilation: getCompiler().__internal__get_compilation()!,
						targetPath,
						outputPath,
						get source() {
							return getCompiler()
								.__internal__get_compilation()!
								.getAsset(filename)?.source!;
						},
						get content() {
							return this.source?.buffer();
						}
					});
				};
			}
		)
	};
};
