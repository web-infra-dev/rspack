import fs from 'node:fs';
import path from 'node:path';

import { escapeSep } from '.';

export const isDirectory = (p: string) => fs.lstatSync(p).isDirectory();
export const isFile = (p: string) => fs.lstatSync(p).isFile();
export const isValidCaseDirectory = (name: string) =>
  !name.startsWith('_') && !name.startsWith('.') && name !== 'node_modules';

export function describeByWalk(
  /**
   * The test file absolute path.
   */
  testFile: string,
  createCase: (name: string, src: string, dist: string) => void,
  options: {
    type?: 'file' | 'directory';
    level?: number;
    source?: string;
    dist?: string;
    absoluteDist?: boolean;
    describe?: Describe;
    exclude?: RegExp[];
  } = {},
) {
  const describeFn = options.describe || describe;
  const testBasename = path
    .basename(testFile)
    .replace(/(\.part\d+)?\.(diff|hot)?test\.(j|t)s/, '');
  const testId = testBasename.charAt(0).toLowerCase() + testBasename.slice(1);
  const sourceBase =
    options.source || path.join(path.dirname(testFile), `${testId}Cases`);

  const testSourceId = path.basename(sourceBase);
  const absoluteTestDir = path.join(testFile, '..');

  const distBase =
    options.dist || path.join(path.dirname(testFile), 'js', testId);
  const level = options.level || 2;
  const type = options.type || 'directory';
  const absoluteDist = options.absoluteDist ?? true;
  function describeDirectory(dirname: string, currentLevel: number) {
    fs.readdirSync(path.join(sourceBase, dirname))
      .filter(isValidCaseDirectory)
      .filter((folder) => {
        if (options.exclude) {
          if (
            options.exclude.some((exclude) => {
              return exclude.test(
                path.join(dirname, folder).replace(/\\/g, '/'),
              );
            })
          ) {
            return false;
          }
        }

        return true;
      })
      .filter((folder) => {
        const p = path.join(sourceBase, dirname, folder);
        if (type === 'file' && currentLevel === 1) {
          return isFile(p);
        }
        if (type === 'directory' || currentLevel > 1) {
          return isDirectory(p);
        }
        return false;
      })
      .map((folder) => {
        const caseName = path.join(dirname, folder);
        if (currentLevel > 1) {
          describeDirectory(caseName, currentLevel - 1);
        } else {
          const name = escapeSep(
            path
              .join(`${testId}Cases-${testSourceId}`, caseName)
              .split('.')
              .shift()!,
          );
          let suiteName = name;

          // support filter test by absolute path
          if (process.env.testFilter?.includes(absoluteTestDir)) {
            const absoluteName = path.join(
              absoluteTestDir,
              testSourceId,
              caseName,
            );
            if (absoluteName.includes(process.env.testFilter!)) {
              suiteName = absoluteName;
            }
          }

          describeFn(suiteName, () => {
            const source = path.join(sourceBase, caseName);
            let dist = '';
            if (absoluteDist) {
              dist = path.join(distBase, caseName);
            } else {
              const relativeDist = options.dist || 'dist';
              if (path.isAbsolute(relativeDist)) {
                dist = path.join(relativeDist, caseName);
              } else {
                dist = path.join(sourceBase, caseName, relativeDist);
              }
            }
            createCase(suiteName, source, dist);
          });
        }
      });
  }

  describeFn(testId, () => {
    describeDirectory('', level);
  });
}
