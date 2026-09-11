import { createRequire } from 'node:module';
import type { Compiler } from '../Compiler';

const require = createRequire(import.meta.url);
const modules = new Map<string, unknown>();
const promises = new Map<string, Promise<unknown>>();
const urls = new Map<string, string>();

/** Shared CJS/ESM loading; export validation belongs to the caller. */
export function loadModule(
  path: string,
  type: string | undefined,
  compiler: Pick<Compiler, '__internal_browser_require'> | undefined,
  callback: (error?: unknown, module?: unknown) => void,
): void {
  if (IS_BROWSER) {
    let module: unknown;
    try {
      if (!compiler)
        throw new Error('workerFunction is not supported in browsers');
      module = compiler.__internal_browser_require(path);
    } catch (error) {
      return callback(error);
    }
    return callback(undefined, module);
  }
  const key = `${type ?? 'commonjs'}\0${path}`;
  if (modules.has(key)) return callback(undefined, modules.get(key));
  if (type === 'module') {
    let promise = promises.get(key);
    if (!promise) {
      let url = urls.get(path);
      if (!url) {
        url = (require('node:url') as typeof import('node:url'))
          .pathToFileURL(path)
          .toString();
        urls.set(path, url);
      }
      promise = import(url).then(
        (module) => {
          modules.set(key, module);
          promises.delete(key);
          return module;
        },
        (error) => {
          promises.delete(key);
          throw error;
        },
      );
      promises.set(key, promise);
    }
    void promise.then((module) => callback(undefined, module), callback);
  } else {
    let module: unknown;
    try {
      module = require(path);
    } catch (error) {
      if (
        error instanceof Error &&
        (error as NodeJS.ErrnoException).code === 'EMFILE'
      ) {
        setImmediate(() => loadModule(path, type, compiler, callback));
        return;
      }
      return callback(error);
    }
    callback(undefined, module);
  }
}
