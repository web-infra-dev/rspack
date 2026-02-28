import path from 'node:path';

export function escapeSep(str: string) {
  return str.split(path.win32.sep).join(path.posix.sep);
}
