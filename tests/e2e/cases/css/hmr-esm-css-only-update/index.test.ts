import { test, expect } from '@/fixtures';

const MAIN_A = 'rgb(163, 255, 255)';
const MAIN_B = 'rgb(0, 128, 0)';
const FEATURE_A = 'rgb(200, 0, 0)';
const FEATURE_B = 'rgb(0, 0, 200)';

// css-only edits under `chunkLoading: 'import'`: no js module changes, the
// update carries only the miniCss manifest entry and the existing link is
// swapped in place (the window marker rules out a full reload)
test('editing the entry stylesheet alone applies without a reload', async ({
  page,
  fileAction,
}) => {
  const body = page.locator('body');
  await expect(page.locator('#root')).toHaveText('step-0');
  await expect(body).toHaveCSS('background-color', MAIN_A);
  await page.evaluate(() => {
    (window as { __e2eAlive?: boolean }).__e2eAlive = true;
  });

  fileAction.updateFile('src/main.css', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );

  await expect(body).toHaveCSS('background-color', MAIN_B);
  expect(
    await page.evaluate(() => (window as { __e2eAlive?: boolean }).__e2eAlive),
  ).toBe(true);
});

test('editing the async chunk stylesheet alone applies without a reload', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveText('step-0');
  await expect(root).toHaveCSS('color', FEATURE_A);
  await page.evaluate(() => {
    (window as { __e2eAlive?: boolean }).__e2eAlive = true;
  });

  fileAction.updateFile('src/feature.css', (content) =>
    content.replace('rgb(200, 0, 0)', 'rgb(0, 0, 200)'),
  );

  await expect(root).toHaveCSS('color', FEATURE_B);
  expect(
    await page.evaluate(() => (window as { __e2eAlive?: boolean }).__e2eAlive),
  ).toBe(true);
});
