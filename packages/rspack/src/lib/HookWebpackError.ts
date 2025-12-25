/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/HookWebpackError.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Callback } from '@rspack/lite-tapable';
import WebpackError from './WebpackError';

export class HookWebpackError extends WebpackError {
  hook: string;
  error: Error;

  /**
   * Creates an instance of HookWebpackError.
   * @param error inner error
   * @param hook name of hook
   */
  constructor(error: Error, hook: string) {
    super(error.message);

    this.name = 'HookWebpackError';
    this.hook = hook;
    this.error = error;
    this.hideStack = true;
    this.details = `caused by plugins in ${hook}\n${error.stack}`;

    this.stack += `\n-- inner error --\n${error.stack}`;
  }
}

export default HookWebpackError;

/**
 * @param error an error
 * @param hook name of the hook
 * @returns a webpack error
 */
export const makeWebpackError = (error: Error, hook: string): WebpackError => {
  if (error instanceof WebpackError) return error;
  return new HookWebpackError(error, hook);
};

/**
 * @param callback webpack error callback
 * @param hook name of hook
 * @returns generic callback
 */
export const makeWebpackErrorCallback = <T>(
  callback: (error?: WebpackError | null, result?: T) => void,
  hook: string,
): Callback<Error, T> => {
  return (err, result) => {
    if (err) {
      if (err instanceof WebpackError) {
        callback(err);
        return;
      }
      callback(new HookWebpackError(err, hook));
      return;
    }
    callback(null, result);
  };
};

/**
 * @param fn function which will be wrapping in try catch
 * @param hook name of hook
 * @returns the result
 */
export const tryRunOrWebpackError = <T>(fn: () => T, hook: string): T => {
  let r: T;
  try {
    r = fn();
  } catch (err) {
    if (err instanceof WebpackError) {
      throw err;
    }
    throw new HookWebpackError(err as Error, hook);
  }
  return r;
};
