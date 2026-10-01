import path from 'node:path';
import type { RspackOptions } from '@rspack/core';

interface TestCase {
  usedExports: string[];
  expect: Record<string, string[] | boolean>;
  name?: string;
}

export default (testCases: Record<string, TestCase>): RspackOptions[] => {
  const configs: RspackOptions[] = [];
  for (const name of Object.keys(testCases)) {
    const testCase = testCases[name];
    testCase.name = name;
    const entry = `../_helpers/entryLoader.mjs?${JSON.stringify(testCase)}!`;
    const resolve: { alias: Record<string, string> } = {
      alias: {},
    };
    let i = 0;
    for (const file of Object.keys(testCase.expect)) {
      resolve.alias[file] =
        path.join(import.meta.dirname, 'inner-file.js') + '?' + i++;
    }
    configs.push({
      name: `${name} without module concatenation`,
      mode: 'production',
      entry,
      resolve,
      optimization: {
        concatenateModules: false,
      },
    });
    configs.push({
      name: `${name} with module concatenation`,
      mode: 'production',
      entry,
      resolve,
    });
  }
  return configs;
};
