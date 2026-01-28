import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { isEsmFile } from './isEsmFile';

const require = createRequire(import.meta.url);

export const crossImport = async <T = any>(path: string): Promise<T> => {
  if (isEsmFile(path)) {
    const url = pathToFileURL(path).href;
    const { default: config } = await import(url);
    return config;
  }
  let result = require(path);
  // compatible with export default config in common ts config
  if (result && typeof result === 'object' && 'default' in result) {
    result = result.default || {};
  }
  return result;
};
