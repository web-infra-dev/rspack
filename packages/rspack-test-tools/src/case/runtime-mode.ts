import rspack, { type RspackOptions } from '@rspack/core';

const runtimeModeDefine = {
  'globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK': JSON.stringify(true),
};

export function applyRuntimeModeTestDefines(options: RspackOptions) {
  if (options.experiments?.runtimeMode !== 'rspack') {
    return;
  }

  options.plugins ??= [];
  options.plugins.push(new rspack.DefinePlugin(runtimeModeDefine));
}
