import fs from 'node:fs';
import path from 'node:path';
import { expect, test as base } from '@/fixtures';
import { calcPathInfo } from '@/fixtures/pathInfo';

for (const exportKind of ['named', 'default']) {
  const test = base.extend({
    pathInfo: async ({ task }, use) => {
      const pathInfo = await calcPathInfo(
        task.filepath!,
        process.env.RSTEST_WORKER_ID!,
      );
      if (exportKind === 'default') {
        const entry = path.join(pathInfo.tempProjectDir, 'src/index.js');
        fs.writeFileSync(
          entry,
          fs
            .readFileSync(entry, 'utf8')
            .replace('import { create }', 'import create'),
        );
        const barrel = path.join(pathInfo.tempProjectDir, 'src/lib/index.js');
        fs.writeFileSync(
          barrel,
          fs
            .readFileSync(barrel, 'utf8')
            .replace('export const create =', 'export default')
            // Star fallback could eagerly expand the barrel even without local-default detection.
            .replace('export *', 'export { createStore }'),
        );
      }
      try {
        await use(pathInfo);
      } finally {
        fs.rmSync(pathInfo.tempProjectDir, { recursive: true, force: true });
      }
    },
  });

  test(`activating a lazy reexport does not reload a barrel with a ${exportKind} local export`, async ({
    page,
  }) => {
    await expect(page.locator('body')).toHaveAttribute('data-infra', 'initial');

    await page.getByText('Activate feature').click();
    await expect(page.locator('body')).toHaveAttribute(
      'data-feature',
      'feature',
    );

    expect(
      await page.evaluate(() => sessionStorage.getItem('documentLoads')),
    ).toBe('1');
  });
}
