/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/WebpackError.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { inspect } from 'node:util';
import type { DependencyLocation } from '@rspack/binding';
import type { Chunk } from '../Chunk';
import type { Module } from '../Module';

export class WebpackError extends Error {
  loc?: DependencyLocation;
  file?: string;
  chunk?: Chunk;
  module?: null | Module;
  details?: string;
  hideStack?: boolean;
}

Object.defineProperty(WebpackError.prototype, inspect.custom, {
  value: function () {
    return this.stack + (this.details ? `\n${this.details}` : '');
  },
  enumerable: false,
  configurable: true,
});

export default WebpackError;
