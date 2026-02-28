import path from 'node:path';

export function isJavaScript(filePath: string): boolean {
  const ext = path.extname(filePath).toLowerCase();
  return ext === '.js' || ext === '.mjs';
}

export function isCss(filePath: string): boolean {
  return path.extname(filePath).toLowerCase() === '.css';
}
