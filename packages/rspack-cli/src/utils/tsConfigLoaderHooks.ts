import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { compileTypeScript } from './compileTypeScript';
import { isEsmFile } from './isEsmFile';
import isTsFile from './isTsFile';

const getModuleType = (filename: string) =>
  isEsmFile(filename) ? 'es6' : 'commonjs';

export async function load(
  url: string,
  _context: { format?: string | null },
  nextLoad: (
    url: string,
    context: { format?: string | null },
  ) => Promise<{ format: string; source?: string | Buffer | null }>,
) {
  if (!url.startsWith('file:')) {
    return nextLoad(url, _context);
  }

  const filename = fileURLToPath(url);
  if (!isTsFile(filename)) {
    return nextLoad(url, _context);
  }

  const moduleType = getModuleType(filename);
  const format = moduleType === 'es6' ? 'module' : 'commonjs';
  const source = await readFile(new URL(url), 'utf8');

  try {
    return {
      format,
      shortCircuit: true,
      source: compileTypeScript(source, filename, moduleType),
    };
  } catch (err) {
    throw new Error(
      `Failed to transform file "${filename}" when loading TypeScript config file:\n ${err instanceof Error ? err.message : String(err)}`,
    );
  }
}
