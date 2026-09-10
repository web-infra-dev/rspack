import path from 'node:path';
import fs from 'fs-extra';
import type { Fixtures } from 'rstack/test';

type PathInfo = {
  testFile: string;
  testProjectDir: string;
  tempProjectDir: string;
};

export type PathInfoFixtures = {
  pathInfo: PathInfo;
};

const tempDir = path.resolve(import.meta.dirname, '../temp');
export async function calcPathInfo(
  testFile: string,
  workerId: string,
): Promise<PathInfo> {
  const testProjectDir = path.dirname(testFile);
  const isRspackConfigExist = await fs.exists(
    path.join(testProjectDir, 'rspack.config.js'),
  );
  if (!isRspackConfigExist) {
    throw new Error(`rspack config not exist in ${testProjectDir}`);
  }

  // Native ESM modules are cached by URL. Give each test a fresh module graph.
  await fs.ensureDir(tempDir);
  const tempProjectDir = await fs.mkdtemp(path.join(tempDir, `${workerId}-`));
  await fs.copy(testProjectDir, tempProjectDir);

  return {
    testFile,
    testProjectDir,
    tempProjectDir,
  };
}

export const pathInfoFixtures: Fixtures<PathInfoFixtures> = {
  pathInfo: async ({ task }, use) => {
    const pathInfo: PathInfo = await calcPathInfo(
      task.filepath!,
      process.env.RSTEST_WORKER_ID!,
    );
    await use(pathInfo);
    await fs.remove(pathInfo.tempProjectDir);
  },
};
