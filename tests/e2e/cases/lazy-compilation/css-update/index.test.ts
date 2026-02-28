import { expect, test } from '@/fixtures';

test('should update style', async ({ page }) => {
  const body = await page.$('body');
  const backgroundColor = await body!.evaluate(
    (el) => window.getComputedStyle(el).backgroundColor,
  );
  // first time enter the page, style is red
  expect(backgroundColor, 'red');

  // second time enter the page, this time brings query,
  // trigger lazy-compile
  const url = await body!.evaluate(() => window.location.href);
  await page.goto(`${url}?1`);
  const updatedBody = await page.$('body');
  const updatedBackgroundColor = await updatedBody!.evaluate(
    (el) => window.getComputedStyle(el).backgroundColor,
  );
  // first time enter the page, style is red
  expect(updatedBackgroundColor, 'blue');
});
