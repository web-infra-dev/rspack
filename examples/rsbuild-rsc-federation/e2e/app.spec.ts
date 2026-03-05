import { expect, test } from '@playwright/test';

const appUrls = [
  { name: 'host', url: 'http://localhost:3330/' },
  { name: 'remote-copy', url: 'http://localhost:3331/' },
];

for (const app of appUrls) {
  test(`renders ${app.name} demo app and supports interaction`, async ({
    page,
  }) => {
    await page.goto(app.url);

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
}
