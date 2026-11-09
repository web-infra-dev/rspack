import binding, { type BuildInfo, type Module } from '@rspack/binding';
import type { NormalModule } from '../NormalModule';
import { SourceAdapter } from '../util/source';
import { pureLoaderError } from './utils';

/** The worker API deliberately excludes compilation graph access. */
export function createWorkerModule(native: Module, buildInfo?: BuildInfo) {
  let active = true;
  const getNative = () => {
    if (!active)
      throw new Error(
        'Worker Module is unavailable after its loader task has completed',
      );
    return native;
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
      getNative();
      if (!buildInfo)
        throw pureLoaderError('Module buildInfo (including asset reads)');
      // Custom values may include functions. Keep the existing object bridge,
      // including assets, whose reflector belongs to the main JS env.
      return buildInfo;
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
  return {
    module: module as unknown as NormalModule,
    dispose() {
      active = false;
    },
  };
}
