import { expect, test } from '@/fixtures';

test('activating a lazy reexport does not reload the page', async ({
  page,
}) => {
  await expect(page.locator('body')).toHaveAttribute('data-infra', 'initial');

  await page.getByText('Activate feature').click();
  await expect(page.locator('body')).toHaveAttribute('data-feature', 'feature');

  expect(
    await page.evaluate(() => sessionStorage.getItem('documentLoads')),
  ).toBe('1');
});
