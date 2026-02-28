import path from 'node:path';
export const TS_EXTENSION = ['.ts', '.cts', '.mts'];
const isTsFile = (configPath: string) => {
  const ext = path.extname(configPath);
  return TS_EXTENSION.includes(ext);
};

export default isTsFile;
