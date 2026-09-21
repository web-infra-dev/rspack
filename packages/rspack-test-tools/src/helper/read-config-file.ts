import path from 'node:path';
import { isModuleNamespaceObject } from 'node:util/types';
import type { RspackOptions } from '@rspack/core';
import fs from 'fs-extra';
import { DEBUG_SCOPES } from '../test/debug';
import type { ITestContext } from '../type';

export const RSPACK_CONFIG_FILES = [
  'rspack.config.ts',
  'rspack.config.mjs',
  'rspack.config.cjs',
  'rspack.config.js',
];

export function findRspackConfigFile(dir: string): string | undefined {
  return RSPACK_CONFIG_FILES.map((file) => path.join(dir, file)).find((file) =>
    fs.existsSync(file),
  );
}

export function readConfigFile(
  files: string[],
  context: ITestContext,
  prevOption?: RspackOptions,
  functionApply?: (
    config: (RspackOptions | ((...args: unknown[]) => RspackOptions))[],
  ) => RspackOptions[],
): RspackOptions[] {
  const existsFile = files.find((i) => fs.existsSync(i));
  let fileConfig = existsFile ? require(existsFile) : {};

  if (existsFile?.endsWith('.mjs') || isModuleNamespaceObject(fileConfig)) {
    fileConfig = fileConfig.default;
  }

  if (typeof fileConfig === 'function') {
    fileConfig = fileConfig(
      { config: prevOption },
      { testPath: context.getDist(), tempPath: context.getTemp() },
    );
  }
  const configArr = Array.isArray(fileConfig) ? fileConfig : [fileConfig];
  if (existsFile) {
    context.setValue(DEBUG_SCOPES.CompilerOptionsReadConfigFile, {
      file: existsFile,
      config: fileConfig,
    });
  }
  return functionApply ? functionApply(configArr) : configArr;
}
