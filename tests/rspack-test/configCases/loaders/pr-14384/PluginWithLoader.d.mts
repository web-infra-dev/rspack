import type { Compiler } from '@rspack/core';

export default class PluginWithLoader {
  apply(compiler: Compiler): void;
}
