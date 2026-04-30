import type { RspackOptions } from '@rspack/core';
import fs from 'fs-extra';
import { DEBUG_SCOPES } from '../test/debug';
import type { ITestContext } from '../type';

function isNodeEsmNamespaceObject(value: unknown): value is {
  default: RspackOptions | ((...args: unknown[]) => RspackOptions);
} {
  if (!value || (typeof value !== 'object' && typeof value !== 'function')) {
    return false;
  }

  if (!Object.prototype.hasOwnProperty.call(value, 'default')) {
    return false;
  }

  return (
    (value as { [Symbol.toStringTag]?: unknown })[Symbol.toStringTag] ===
    'Module'
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

  if (isNodeEsmNamespaceObject(fileConfig)) {
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
