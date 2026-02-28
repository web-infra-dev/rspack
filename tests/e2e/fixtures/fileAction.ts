import path from 'node:path';
import fs from 'fs-extra';
import type { Fixtures } from '@playwright/test';
import type { RspackFixtures } from './rspack';

type FileAction = {
  renameFile(oldPath: string, newPath: string): void;
  updateFile(relativePath: string, fn: (content: string) => string): void;
  deleteFile(relativePath: string): void;
};

type FileActionFixtures = {
  fileAction: FileAction;
};

export const fileActionFixtures: Fixtures<
  FileActionFixtures,
  {},
  RspackFixtures
> = {
  fileAction: async ({ rspack }, use) => {
    await use({
      renameFile(oldPath, newPath) {
        const oldFilePath = path.resolve(rspack.projectDir, oldPath);
        const newFilePath = path.resolve(rspack.projectDir, newPath);
        fs.renameSync(oldFilePath, newFilePath);
      },
      updateFile(relativePath, fn) {
        const filePath = path.resolve(rspack.projectDir, relativePath);
        const fileExists = fs.existsSync(filePath);
        const content = fileExists ? fs.readFileSync(filePath).toString() : '';

        fs.writeFileSync(filePath, fn(content));
      },
      deleteFile(relativePath) {
        const filePath = path.resolve(rspack.projectDir, relativePath);
        const fileExists = fs.existsSync(filePath);
        if (!fileExists) {
          return;
        }

        fs.unlinkSync(filePath);
      },
    });
  },
};
