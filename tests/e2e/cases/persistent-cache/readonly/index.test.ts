import { expect, test } from '@/fixtures';
import fs from 'fs';
import path from 'path';

async function expectContent(page: any, data: string) {
  await expect(async () => {
    await page.reload();
    expect(await page.locator('div').innerText()).toBe(data);
  }).toPass();
}

function getCacheFiles(cacheDir: string): Map<string, number> {
  const mtimes = new Map<string, number>();

  if (!fs.existsSync(cacheDir)) {
    console.warn(`no cache found at ${cacheDir}`);
    return mtimes;
  }

  function walk(dir: string) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
      const fullPath = path.join(dir, file);
      const stat = fs.statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else {
        const relativePath = path.relative(cacheDir, fullPath);
        mtimes.set(relativePath, stat.mtimeMs);
      }
    }
  }

  walk(cacheDir);
  return mtimes;
}

test('readonly cache should not write to disk', async ({
  page,
  fileAction,
  rspack,
  pathInfo,
}, testInfo) => {
  // chromium incremental modifies the passed in cache storage location
  // https://github.com/web-infra-dev/rspack/blob/85b1a7e0238e73b8d289aea7e9ae8018ed4bf2b0/tests/e2e/playwright.config.ts#L68-L74
  test.skip(
    testInfo.project.name === 'chromium-incremental',
    'Test not compatible with chromium-incremental cache setup',
  );
  const cacheDir = (rspack as any).config.cache.storage.directory;
  await expectContent(page, 'initial');

  await new Promise((res) => setTimeout(res, 500));

  const initialMtimes = getCacheFiles(cacheDir);
  expect(initialMtimes.size, 'cache was written').toBeGreaterThan(0);

  await rspack.stop();
  await new Promise((res) => setTimeout(res, 500));

  fileAction.updateFile('module.js', () => "export const value = 'modified';");

  // Modify the rspack config directly (editing the file doesn't work since config is already loaded)
  // Access private config property to enable readonly mode
  (rspack as any).config.cache.readonly = true;

  await rspack.start();
  await expectContent(page, 'modified');
  await new Promise((res) => setTimeout(res, 500));

  const finalMtimes = getCacheFiles(cacheDir);
  for (const [file, initialMtime] of initialMtimes.entries()) {
    const finalMtime = finalMtimes.get(file);
    expect(
      finalMtime,
      `cache file (${file}) was not modified during rebuild`,
    ).toBe(initialMtime);
  }

  expect(finalMtimes.size, 'same number of cache files are present').toBe(
    initialMtimes.size,
  );

  await rspack.stop();
});
