import { builtinModules } from 'node:module';
import type { RspackOptions } from '@rspack/core';

const nodeBuiltinExternals = Object.fromEntries(
  builtinModules.flatMap((module) => {
    const normalized = module.replace(/^node:/, '');
    const externals = [
      [normalized, `node-commonjs ${normalized}`],
      [`node:${normalized}`, `node-commonjs node:${normalized}`],
    ];
    return normalized === 'test'
      ? externals.filter(([request]) => request !== 'test')
      : externals;
  }),
);

export function applyNodeBuiltinExternals(options: RspackOptions) {
  const { externals } = options;
  if (!externals) {
    options.externals = nodeBuiltinExternals;
  } else if (Array.isArray(externals)) {
    options.externals = [...externals, nodeBuiltinExternals];
  } else {
    options.externals = [externals, nodeBuiltinExternals];
  }
}
