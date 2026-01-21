import { expect, test } from '@/fixtures';

test('async startup should not preload remote component chunk', async ({
  page,
}) => {
  await page.waitForSelector('button:has-text("Load Remote")');

  const initialResources = await page.evaluate(() =>
    performance.getEntriesByType('resource').map((entry) => entry.name),
  );
  const hasRemoteComponent = initialResources.some((name) =>
    name.includes('RemoteComponent'),
  );
  expect(hasRemoteComponent).toBe(false);

  const responsePromise = page.waitForResponse(
    (response) =>
      response.url().includes('RemoteComponent') &&
      response.request().method() === 'GET',
    { timeout: 10000 },
  );

  await page.getByText('Load Remote').click();

  await responsePromise;
  await page.waitForSelector('p:has-text("Remote Component")');
  expect(await page.getByText('Remote Component').count()).toBe(1);
});
