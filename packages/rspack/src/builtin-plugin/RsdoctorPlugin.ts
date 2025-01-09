import { BuiltinPluginName, JsRsdoctorAsset, JsRsdoctorChunkGraph, JsRsdoctorModuleGraph, JsRsdoctorModuleSource, RawRsdoctorPluginOptions } from "@rspack/binding";
import { z } from "zod";
import { create } from "./base";
import { validate } from "../util/validate";
import { Compiler } from "../Compiler";
import * as liteTapable from "@rspack/lite-tapable";
import { Compilation } from "../Compilation";

export type RsdoctorRspackPluginOptions = {};
const rsdoctorPluginSchema = z.strictObject({}) satisfies z.ZodType<RsdoctorRspackPluginOptions>;

const RsdoctorRspackPluginImpl = create(
  BuiltinPluginName.HtmlRspackPlugin,
  function (
    this: Compiler,
    c: RsdoctorRspackPluginOptions = {}
  ): RawRsdoctorPluginOptions {
    validate(c, rsdoctorPluginSchema);
    return {};
  }
);

export type RsdoctorRspackPluginHooks = {
  moduleGraph: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorModuleGraph], false | void
  >;
  chunkGraph: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorChunkGraph], false | void
  >;
  moduleSources: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorModuleSource[]], false | void
  >;
  assets: liteTapable.AsyncSeriesBailHook<
    [JsRsdoctorAsset[]], false | void
  >;
};

const compilationHooksMap: WeakMap<Compilation, RsdoctorRspackPluginHooks> =
  new WeakMap();

const RsdoctorRspackPlugin = RsdoctorRspackPluginImpl as typeof RsdoctorRspackPluginImpl & {
  /**
   * @deprecated Use `getCompilationHooks` instead.
   */
  getHooks: (compilation: Compilation) => RsdoctorRspackPluginHooks;
  getCompilationHooks: (compilation: Compilation) => RsdoctorRspackPluginHooks;
};

RsdoctorRspackPlugin.getHooks = RsdoctorRspackPlugin.getCompilationHooks = (
  compilation: Compilation
) => {
  if (!(compilation instanceof Compilation)) {
    throw new TypeError(
      "The 'compilation' argument must be an instance of Compilation"
    );
  }
  let hooks = compilationHooksMap.get(compilation);
  if (hooks === undefined) {
    hooks = {
      moduleGraph: new liteTapable.AsyncSeriesBailHook<[JsRsdoctorModuleGraph], false | void>(["moduleGraph"]),
      chunkGraph: new liteTapable.AsyncSeriesBailHook<[JsRsdoctorChunkGraph], false | void>(["chunkGraph"]),
      moduleSources: new liteTapable.AsyncSeriesBailHook<[JsRsdoctorModuleSource[]], false | void>(["moduleSources"]),
      assets: new liteTapable.AsyncSeriesBailHook<[JsRsdoctorAsset[]], false | void>(["assets"]),
    };
    compilationHooksMap.set(compilation, hooks);
  }
  return hooks;
};

export { RsdoctorRspackPlugin };