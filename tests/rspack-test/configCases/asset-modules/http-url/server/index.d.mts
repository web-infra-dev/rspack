import type { Compiler } from '@rspack/core';

export default class ServerPlugin {
  constructor(port: number);
  apply(compiler: Compiler): void;
}
