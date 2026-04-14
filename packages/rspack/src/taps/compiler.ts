import binding from '@rspack/binding';
import type { CreatePartialRegisters } from './types';

export const createCompilerHooksRegisters: CreatePartialRegisters<
  `Compiler`
> = (getCompiler, createTap) => {
  return {
    registerCompilerThisCompilationTaps: createTap(
      binding.CompilerHooks.ThisCompilation,

      function () {
        return getCompiler().hooks.thisCompilation;
      },

      function (queried) {
        return function (native: binding.JsCompilation) {
          getCompiler().__internal__create_compilation(native);
          return queried.call(
            getCompiler().__internal__get_compilation()!,
            getCompiler().__internal__get_compilation_params()!,
          );
        };
      },
    ),
    registerCompilerCompilationTaps: createTap(
      binding.CompilerHooks.Compilation,

      function () {
        return getCompiler().hooks.compilation;
      },

      function (queried) {
        return function () {
          return queried.call(
            getCompiler().__internal__get_compilation()!,
            getCompiler().__internal__get_compilation_params()!,
          );
        };
      },
    ),
    registerCompilerMakeTaps: createTap(
      binding.CompilerHooks.Make,

      function () {
        return getCompiler().hooks.make;
      },

      function (queried) {
        return async function () {
          return queried.promise(getCompiler().__internal__get_compilation()!);
        };
      },
    ),
    registerCompilerFinishMakeTaps: createTap(
      binding.CompilerHooks.FinishMake,

      function () {
        return getCompiler().hooks.finishMake;
      },

      function (queried) {
        return async function () {
          return queried.promise(getCompiler().__internal__get_compilation()!);
        };
      },
    ),
    registerCompilerShouldEmitTaps: createTap(
      binding.CompilerHooks.ShouldEmit,

      function () {
        return getCompiler().hooks.shouldEmit;
      },

      function (queried) {
        return function () {
          return queried.call(getCompiler().__internal__get_compilation()!);
        };
      },
    ),
    registerCompilerEmitTaps: createTap(
      binding.CompilerHooks.Emit,

      function () {
        return getCompiler().hooks.emit;
      },

      function (queried) {
        return async function () {
          return queried.promise(getCompiler().__internal__get_compilation()!);
        };
      },
    ),
    registerCompilerAfterEmitTaps: createTap(
      binding.CompilerHooks.AfterEmit,

      function () {
        return getCompiler().hooks.afterEmit;
      },

      function (queried) {
        return async function () {
          return queried.promise(getCompiler().__internal__get_compilation()!);
        };
      },
    ),
    registerCompilerAssetEmittedTaps: createTap(
      binding.CompilerHooks.AssetEmitted,

      function () {
        return getCompiler().hooks.assetEmitted;
      },

      function (queried) {
        return async function ({
          filename,
          targetPath,
          outputPath,
        }: binding.JsAssetEmittedArgs) {
          return queried.promise(filename, {
            compilation: getCompiler().__internal__get_compilation()!,
            targetPath,
            outputPath,
            get source() {
              const source = getCompiler()
                .__internal__get_compilation()!
                .getAsset(filename)?.source;
              if (!source) {
                throw new Error(`Asset ${filename} not found`);
              }
              return source;
            },
            get content() {
              return this.source?.buffer();
            },
          });
        };
      },
    ),
  };
};
