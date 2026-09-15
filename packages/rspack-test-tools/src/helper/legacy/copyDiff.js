// @ts-nocheck
import fs from 'node:fs';
import path from 'node:path';
import { rimrafSync } from 'rimraf';

export function copyDiff(src, dest, initial) {
  fs.mkdirSync(dest, { recursive: true });
  const files = fs.readdirSync(src);
  for (const filename of files) {
    const srcFile = path.join(src, filename);
    const destFile = path.join(dest, filename);
    const stats = fs.lstatSync(srcFile);
    if (stats.isSymbolicLink()) {
      const linkTarget = fs.readlinkSync(srcFile);
      if (fs.existsSync(destFile)) {
        const destStats = fs.lstatSync(destFile);
        if (destStats.isSymbolicLink()) {
          fs.unlinkSync(destFile);
        } else if (destStats.isDirectory()) {
          rimrafSync(destFile);
        } else {
          fs.unlinkSync(destFile);
        }
      }
      fs.symlinkSync(linkTarget, destFile);
    } else if (stats.isDirectory()) {
      copyDiff(srcFile, destFile, initial);
    } else {
      const content = fs.readFileSync(srcFile);
      if (/^DELETE\s*$/.test(content.toString('utf-8'))) {
        fs.unlinkSync(destFile);
      } else if (/^DELETE_DIRECTORY\s*$/.test(content.toString('utf-8'))) {
        rimrafSync(destFile);
      } else {
        fs.writeFileSync(destFile, content);
        if (initial) {
          const longTimeAgo = Date.now() - 1000 * 60 * 60 * 24;
          fs.utimesSync(
            destFile,
            Date.now() - longTimeAgo,
            Date.now() - longTimeAgo,
          );
        }
      }
    }
  }
}
