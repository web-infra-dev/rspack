import binding, { type BuildInfo, type Module } from '@rspack/binding';
import type { NormalModule } from '../NormalModule';
import {
  pickCustomBuildInfoFields,
  replaceCustomBuildInfoFields,
} from '../BuildInfo';
import { SourceAdapter } from '../util/source';
import {
  BUILD_INFO_ASSETS_SYMBOL,
  COMMIT_CUSTOM_FIELDS_SYMBOL,
} from '../util/symbols';
import type { WorkerLoaderContext } from './service';
import { strictLoaderError } from './utils';

function restrictBuildInfo(buildInfo: BuildInfo): BuildInfo {
  const assets = {
    configurable: false,
    get() {
      throw strictLoaderError('Reading Module buildInfo.assets');
    },
  };
  return Object.defineProperties(buildInfo, {
    assets,
    [BUILD_INFO_ASSETS_SYMBOL]: assets,
  });
}

export function createWorkerModule(
  native: Module,
  main?: Pick<
    WorkerLoaderContext,
    'cacheBuildInfo' | 'customBuildInfo' | 'updateCustomBuildInfo'
  >,
) {
  let active = true;
  let localBuildInfo: BuildInfo | undefined;
  let hadCustomBuildInfo = false;
  const getNative = () => {
    if (!active)
      throw new Error(
        'Worker Module is unavailable after its loader task has completed',
      );
    return native;
  };
  const getBuildInfo = () => {
    const handle = getNative();
    if (!localBuildInfo) {
      const info = binding.moduleGetBuildInfo(handle);
      if (main) {
        if (main.customBuildInfo) {
          replaceCustomBuildInfoFields(info, main.customBuildInfo);
        }
        hadCustomBuildInfo =
          Object.keys(pickCustomBuildInfoFields(info)).length > 0;
        localBuildInfo = info;
      } else {
        localBuildInfo = restrictBuildInfo(info);
      }
    }
    return localBuildInfo;
  };
  const module = {
    get type() {
      return binding.moduleGetType(getNative());
    },
    get context() {
      return binding.moduleGetContext(getNative());
    },
    get resource() {
      return binding.normalModuleGetResource(getNative());
    },
    get request() {
      return binding.normalModuleGetRequest(getNative());
    },
    get userRequest() {
      return binding.normalModuleGetUserRequest(getNative());
    },
    get rawRequest() {
      return binding.normalModuleGetRawRequest(getNative());
    },
    get matchResource() {
      return binding.normalModuleGetMatchResource(getNative());
    },
    set matchResource(value) {
      binding.normalModuleSetMatchResource(getNative(), value);
    },
    get layer() {
      return binding.moduleGetLayer(getNative());
    },
    get useSourceMap() {
      return binding.moduleGetUseSourceMap(getNative());
    },
    get useSimpleSourceMap() {
      return binding.moduleGetUseSimpleSourceMap(getNative());
    },
    get factoryMeta() {
      return binding.moduleGetFactoryMeta(getNative());
    },
    set factoryMeta(value) {
      binding.moduleSetFactoryMeta(getNative(), value);
    },
    get buildInfo() {
      return getBuildInfo();
    },
    identifier() {
      return binding.moduleGetIdentifier(getNative());
    },
    readableIdentifier() {
      return binding.moduleGetReadableIdentifier(getNative());
    },
    nameForCondition() {
      return binding.moduleGetNameForCondition(getNative());
    },
    originalSource() {
      const source = binding.moduleGetOriginalSource(getNative());
      return source ? SourceAdapter.fromBinding(source) : null;
    },
    size(...args: Parameters<Module['size']>) {
      return binding.moduleSize(getNative(), ...args);
    },
    libIdent(...args: Parameters<Module['libIdent']>) {
      return binding.moduleLibIdent(getNative(), ...args);
    },
    emitFile(...[filename, source, assetInfo]: Parameters<Module['emitFile']>) {
      binding.moduleEmitFile(
        getNative(),
        filename,
        SourceAdapter.toBinding(source),
        assetInfo,
      );
    },
  };
  Object.defineProperties(module, {
    resourceResolveData: {
      get: () => binding.normalModuleGetResourceResolveData(getNative()),
    },
    loaders: {
      get: () => binding.normalModuleGetLoaders(getNative()),
    },
    blocks: {
      get: () => binding.moduleGetBlocks(getNative()),
    },
    dependencies: {
      get: () => binding.moduleGetDependencies(getNative()),
    },
    error: {
      get: () => binding.normalModuleGetError(getNative()),
    },
  });
  Object.defineProperty(module, 'buildInfo', {
    set(value: BuildInfo) {
      const handle = getNative();
      if (main) {
        Object.assign(getBuildInfo(), value);
        return;
      }
      binding.moduleSetBuildInfo(handle, value);
      localBuildInfo = undefined;
    },
  });
  return {
    module: module as unknown as NormalModule,
    commitBuildInfo() {
      if (main && localBuildInfo) {
        const fields = pickCustomBuildInfoFields(localBuildInfo);
        if (Object.keys(fields).length > 0 || hadCustomBuildInfo) {
          if (main.cacheBuildInfo) {
            localBuildInfo[COMMIT_CUSTOM_FIELDS_SYMBOL]();
          }
          main.updateCustomBuildInfo(fields);
        }
      }
    },
    dispose() {
      active = false;
    },
  };
}
