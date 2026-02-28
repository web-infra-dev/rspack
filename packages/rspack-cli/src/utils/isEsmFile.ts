import path from 'node:path';
import readPackageUp from './readPackageUp';

export const isEsmFile = (filePath: string) => {
  if (/\.(mjs|mts)$/.test(filePath)) {
    return true;
  }
  if (/\.(cjs|cts)$/.test(filePath)) {
    return false;
  }
  // package.json should be find from configPath root
  const packageJson = readPackageUp(path.dirname(filePath));
  return packageJson?.type === 'module';
};
