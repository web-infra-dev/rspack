import { expect, test } from '@/fixtures';

test('should not rebuild when an active module is reported again', async ({
  page,
  rspack,
}) => {
  const requests: Array<{ body: string; url: string }> = [];
  page.on('request', (request) => {
    if (
      request.method() === 'POST' &&
      request.url().includes('/_rspack/lazy/trigger')
    ) {
      requests.push({
        body: request.postData() || '',
        url: request.url(),
      });
    }
  });

  await page.getByRole('button', { name: 'load lazy module' }).click();
  await expect(page.locator('body')).toHaveText('lazy module loaded');
  expect(requests).toHaveLength(1);

  const buildCount = rspack.compiler.__buildCount;
  const request = requests[0];
  await page.evaluate(async ({ body, url }) => {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'text/plain' },
      body,
    });
    await response.text();
  }, request);

  await new Promise((resolve) => setTimeout(resolve, 1000));
  expect(rspack.compiler.__buildCount).toBe(buildCount);

  const legacyModule = `legacy-module-${Date.now()}`;
  await page.evaluate(
    async ({ url, legacyModule }) => {
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: legacyModule,
      });
      await response.text();
    },
    { url: request.url, legacyModule },
  );

  await expect.poll(() => rspack.compiler.__buildCount).toBe(buildCount + 1);

  await page.evaluate(
    async ({ url, legacyModule }) => {
      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: legacyModule,
      });
      await response.text();
    },
    { url: request.url, legacyModule },
  );

  await new Promise((resolve) => setTimeout(resolve, 1000));
  expect(rspack.compiler.__buildCount).toBe(buildCount + 1);
});
