import binding, {
  type BuildInfo,
  type Module,
  type Dependency,
  type AsyncDependenciesBlock,
} from '@rspack/binding';
import type { NormalModule } from '../NormalModule';
import { SourceAdapter } from '../util/source';
import { BUILD_INFO_ASSETS_SYMBOL } from '../util/symbols';
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

/** Local module data stays on the worker; graph access uses the main-thread bridge. */
export function createWorkerModule(
  native: Module,
  main?: Pick<WorkerLoaderContext, 'buildInfo' | 'getDependency'>,
) {
  const bridgeBuildInfo = main?.buildInfo;
  let active = true;
  let localBuildInfo: BuildInfo | undefined;
  const getNative = () => {
    if (!active)
      throw new Error(
        'Worker Module is unavailable after its loader task has completed',
      );
    return native;
  };
  const wrapDependency = (dependency: Dependency): Dependency => {
    for (const key of ['ids', '_parentModule'] as const) {
      Object.defineProperty(dependency, key, {
        get() {
          getNative();
          return main!.getDependency(dependency.__internal__id)[key];
        },
      });
    }
    return dependency;
  };
  const wrapBlock = (block: AsyncDependenciesBlock): AsyncDependenciesBlock =>
    new Proxy(block, {
      get(target, key) {
        getNative();
        if (key === 'dependencies')
          return target.dependencies.map(wrapDependency);
        if (key === 'blocks') return target.blocks.map(wrapBlock);
        return Reflect.get(target, key, target);
      },
    });
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
      const handle = getNative();
      if (bridgeBuildInfo) return bridgeBuildInfo;
      return (localBuildInfo ??= restrictBuildInfo(
        binding.moduleGetBuildInfo(handle),
      ));
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
      get: () => {
        const blocks = binding.moduleGetBlocks(getNative());
        return main ? blocks.map(wrapBlock) : blocks;
      },
    },
    dependencies: {
      get: () => {
        const dependencies = binding.moduleGetDependencies(getNative());
        return main ? dependencies.map(wrapDependency) : dependencies;
      },
    },
    error: {
      get: () => binding.normalModuleGetError(getNative()),
    },
  });
  Object.defineProperty(module, 'buildInfo', {
    set(value: BuildInfo) {
      const handle = getNative();
      if (bridgeBuildInfo) {
        Object.assign(bridgeBuildInfo, value);
        return;
      }
      binding.moduleSetBuildInfo(handle, value);
      localBuildInfo = restrictBuildInfo(binding.moduleGetBuildInfo(handle));
    },
  });
  return {
    module: module as unknown as NormalModule,
    dispose() {
      active = false;
    },
  };
}
