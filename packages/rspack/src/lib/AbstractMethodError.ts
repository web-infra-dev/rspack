/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/AbstractMethodError.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import WebpackError from './WebpackError';

const CURRENT_METHOD_REGEXP = /at ([a-zA-Z0-9_.]*)/;

/**
 * @param method method name
 * @returns message
 */
function createMessage(method?: string): string {
  return `Abstract method${method ? ` ${method}` : ''}. Must be overridden.`;
}

class Message extends Error {
  constructor() {
    super();
    this.stack = undefined;
    Error.captureStackTrace(this);
    const match = this.stack!.split('\n')[3].match(CURRENT_METHOD_REGEXP);
    this.message = match?.[1] ? createMessage(match[1]) : createMessage();
  }
}

/**
 * Error for abstract method
 * @example
 * class FooClass {
 *     abstractMethod() {
 *         throw new AbstractMethodError(); // error message: Abstract method FooClass.abstractMethod. Must be overridden.
 *     }
 * }
 *
 */
export class AbstractMethodError extends WebpackError {
  constructor() {
    super(new Message().message);
    this.name = 'AbstractMethodError';
  }
}
