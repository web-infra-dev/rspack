import type { Compiler } from '@rspack/core';

export default class TestChildCompilationFailurePlugin {
  constructor(output: import('@rspack/core').Output);
  apply(compiler: Compiler): void;
}
