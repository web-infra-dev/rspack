import binding from '@rspack/binding';
import { JavascriptModulesPlugin } from '../builtin-plugin';
import { createHash } from '../util/createHash';
import { SourceAdapter } from '../util/source';
import type { CreatePartialRegisters } from './types';

export const createJavaScriptModulesHooksRegisters: CreatePartialRegisters<
  `JavascriptModules`
> = (getCompiler, createTap) => {
  return {
    registerJavascriptModulesChunkHashTaps: createTap(
      binding.RegisterJsTapKind.JavascriptModulesChunkHash,

      function () {
        return JavascriptModulesPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).chunkHash;
      },

      function (queried) {
        return function (chunk: binding.Chunk) {
          if (!getCompiler().options.output.hashFunction) {
            throw new Error("'output.hashFunction' cannot be undefined");
          }
          const hash = createHash(getCompiler().options.output.hashFunction!);
          queried.call(chunk, hash);
          let digestResult: Buffer | string;
          if (getCompiler().options.output.hashDigest) {
            digestResult = hash.digest(
              getCompiler().options.output.hashDigest as string,
            );
          } else {
            digestResult = hash.digest();
          }
          return typeof digestResult === 'string'
            ? Buffer.from(digestResult)
            : digestResult;
        };
      },
    ),

    registerJavascriptModulesRenderContentTaps: createTap(
      binding.RegisterJsTapKind.JavascriptModulesRenderContent,

      function () {
        return JavascriptModulesPlugin.getCompilationHooks(
          getCompiler().__internal__get_compilation()!,
        ).renderContent;
      },

      function (queried) {
        return function ({ source, chunk }: binding.JsRenderContentArgs) {
          const original = SourceAdapter.fromBinding(source);
          const result = queried.call(original, { chunk });
          if (!result) {
            throw new Error(
              'JavascriptModulesPlugin error: JavascriptModulesPlugin.getCompilationHooks().renderContent plugins should return something',
            );
          }
          // Sending the source back through the binding rebuilds it from its
          // string form, so keep the Rust side untouched when no plugin
          // replaced it.
          return result === original
            ? undefined
            : SourceAdapter.toBinding(result);
        };
      },
    ),
  };
};
