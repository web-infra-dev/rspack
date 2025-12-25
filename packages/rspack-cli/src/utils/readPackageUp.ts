import fs from 'node:fs';
import path from 'node:path';

const readPackageUp = (cwd = process.cwd()): { type?: 'module' } | null => {
  let currentDir = path.resolve(cwd);
  let packageJsonPath = path.join(currentDir, 'package.json');

  while (!fs.existsSync(packageJsonPath)) {
    const parentDir = path.dirname(currentDir);
    if (parentDir === currentDir) {
      return null;
    }
    currentDir = parentDir;
    packageJsonPath = path.join(currentDir, 'package.json');
  }
  try {
    return JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  } catch {
    return null;
  }
};

export default readPackageUp;
