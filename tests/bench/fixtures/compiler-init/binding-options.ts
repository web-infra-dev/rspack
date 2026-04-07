import type {
  RawCompilerPlatform,
  RegisterJsTaps,
  ThreadsafeNodeFS,
} from '../../helpers/rspack-binding';
import { rspack } from '@rspack/core';
import { getRawOptions } from '../../../../packages/rspack/src/config/adapter';
import rspackConfig from './rspack.config';

const ASYNC_VOID = async () => {};
const ASYNC_UNDEFINED = async () => undefined;

const compiler = rspack(rspackConfig);

export const rawOptions = getRawOptions(compiler.options, compiler);
export const builtinPlugins = compiler.__internal__builtinPlugins;
export const rawCompilerPlatform = compiler.platform as RawCompilerPlatform;

export const noopRegisterJsTaps = {
  registerCompilerThisCompilationTaps: (_stages) => [],
  registerCompilerCompilationTaps: (_stages) => [],
  registerCompilerMakeTaps: (_stages) => [],
  registerCompilerFinishMakeTaps: (_stages) => [],
  registerCompilerShouldEmitTaps: (_stages) => [],
  registerCompilerEmitTaps: (_stages) => [],
  registerCompilerAfterEmitTaps: (_stages) => [],
  registerCompilerAssetEmittedTaps: (_stages) => [],
  registerCompilationBuildModuleTaps: (_stages) => [],
  registerCompilationStillValidModuleTaps: (_stages) => [],
  registerCompilationSucceedModuleTaps: (_stages) => [],
  registerCompilationExecuteModuleTaps: (_stages) => [],
  registerCompilationAdditionalTreeRuntimeRequirementsTaps: (_stages) => [],
  registerCompilationRuntimeRequirementInTreeTaps: (_stages) => [],
  registerCompilationRuntimeModuleTaps: (_stages) => [],
  registerCompilationFinishModulesTaps: (_stages) => [],
  registerCompilationOptimizeModulesTaps: (_stages) => [],
  registerCompilationAfterOptimizeModulesTaps: (_stages) => [],
  registerCompilationOptimizeTreeTaps: (_stages) => [],
  registerCompilationOptimizeChunkModulesTaps: (_stages) => [],
  registerCompilationBeforeModuleIdsTaps: (_stages) => [],
  registerCompilationChunkHashTaps: (_stages) => [],
  registerCompilationChunkAssetTaps: (_stages) => [],
  registerCompilationProcessAssetsTaps: (_stages) => [],
  registerCompilationAfterProcessAssetsTaps: (_stages) => [],
  registerCompilationSealTaps: (_stages) => [],
  registerCompilationAfterSealTaps: (_stages) => [],
  registerNormalModuleFactoryBeforeResolveTaps: (_stages) => [],
  registerNormalModuleFactoryFactorizeTaps: (_stages) => [],
  registerNormalModuleFactoryResolveTaps: (_stages) => [],
  registerNormalModuleFactoryResolveForSchemeTaps: (_stages) => [],
  registerNormalModuleFactoryAfterResolveTaps: (_stages) => [],
  registerNormalModuleFactoryCreateModuleTaps: (_stages) => [],
  registerContextModuleFactoryBeforeResolveTaps: (_stages) => [],
  registerContextModuleFactoryAfterResolveTaps: (_stages) => [],
  registerJavascriptModulesChunkHashTaps: (_stages) => [],
  registerHtmlPluginBeforeAssetTagGenerationTaps: (_stages) => [],
  registerHtmlPluginAlterAssetTagsTaps: (_stages) => [],
  registerHtmlPluginAlterAssetTagGroupsTaps: (_stages) => [],
  registerHtmlPluginAfterTemplateExecutionTaps: (_stages) => [],
  registerHtmlPluginBeforeEmitTaps: (_stages) => [],
  registerHtmlPluginAfterEmitTaps: (_stages) => [],
  registerRuntimePluginCreateScriptTaps: (_stages) => [],
  registerRuntimePluginCreateLinkTaps: (_stages) => [],
  registerRuntimePluginLinkPreloadTaps: (_stages) => [],
  registerRuntimePluginLinkPrefetchTaps: (_stages) => [],
  registerRsdoctorPluginModuleGraphTaps: (_stages) => [],
  registerRsdoctorPluginChunkGraphTaps: (_stages) => [],
  registerRsdoctorPluginModuleIdsTaps: (_stages) => [],
  registerRsdoctorPluginModuleSourcesTaps: (_stages) => [],
  registerRsdoctorPluginAssetsTaps: (_stages) => [],
} satisfies RegisterJsTaps;

export const noopThreadsafeNodeFS = {
  writeFile: ASYNC_VOID,
  removeFile: ASYNC_VOID,
  mkdir: ASYNC_VOID,
  mkdirp: ASYNC_UNDEFINED,
  removeDirAll: ASYNC_UNDEFINED,
  readDir: ASYNC_UNDEFINED,
  readFile: ASYNC_UNDEFINED,
  stat: ASYNC_UNDEFINED,
  lstat: ASYNC_UNDEFINED,
  realpath: ASYNC_UNDEFINED,
  open: ASYNC_UNDEFINED,
  rename: ASYNC_VOID,
  close: ASYNC_VOID,
  write: ASYNC_UNDEFINED,
  writeAll: ASYNC_UNDEFINED,
  read: ASYNC_UNDEFINED,
  readUntil: ASYNC_UNDEFINED,
  readToEnd: ASYNC_UNDEFINED,
  chmod: ASYNC_VOID,
} satisfies ThreadsafeNodeFS;
