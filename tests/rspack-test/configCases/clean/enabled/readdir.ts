import fs from 'node:fs';
import path from 'node:path';

function handlePath(path: string) {
  return path.replace(/\\/g, '/');
}

export default function readDir(from: string): {
  files: string[];
  directories: string[];
} {
  const collectedFiles: string[] = [];
  const collectedDirectories: string[] = [];
  const stack = [from];
  let cursor;

  while ((cursor = stack.pop())) {
    const stat = fs.statSync(cursor);

    if (stat.isDirectory()) {
      const items = fs.readdirSync(cursor);

      if (from !== cursor) {
        const relative = path.relative(from, cursor);
        collectedDirectories.push(handlePath(relative));
      }

      for (let i = 0; i < items.length; i++) {
        stack.push(path.join(cursor, items[i]));
      }
    } else {
      const relative = path.relative(from, cursor);
      collectedFiles.push(handlePath(relative));
    }
  }

  return {
    files: collectedFiles,
    directories: collectedDirectories,
  };
}
