import { expect, test } from '@playwright/test';

test('renders demo app and supports interaction', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByTestId('app-ready')).toBeVisible();
  await expect(page.getByTestId('status-text')).toHaveText(
    'client entry ready',
  );
  await expect(page.getByTestId('component-rendered')).toHaveText(
    'InteractiveClientDemo',
  );
  await expect(page.getByTestId('counter-value')).toHaveText('0');

  await page.getByTestId('increment-button').click();
  await page.getByTestId('increment-button').click();

  await expect(page.getByTestId('counter-value')).toHaveText('2');
});
