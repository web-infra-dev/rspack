import { expect, test } from '@/fixtures';

test('removing a dynamic entry should not produce compile errors', async ({
  page,
  fileAction,
  rspack,
}) => {
  // Activate both entry proxies first so the backend has them in active_modules.
  await page.goto(`http://localhost:${rspack.devServer.options.port}/`);
  await expect(page.locator('#index1')).toBeVisible({ timeout: 30000 });
  await page.goto(
    `http://localhost:${rspack.devServer.options.port}/index2.html`,
  );
  await expect(page.locator('#index2')).toBeVisible({ timeout: 30000 });

  // Remove index2 entry source while index1 stays.
  fileAction.deleteFile('src/index2.js');

  // Wait for the rebuild triggered by file removal to settle.
  await rspack.waitingForBuild();
  // The watcher may schedule a follow-up rebuild for the HMR update; let it run too.
  await new Promise((r) => setTimeout(r, 500));
  await rspack.waitingForBuild();

  const stats = rspack.compiler._lastCompilation
    ?.getStats()
    .toJson({ all: false, errors: true });
  expect(stats?.errors ?? []).toEqual([]);
});
