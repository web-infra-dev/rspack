import { test, expect } from '@playwright/test';

test('@rspack/browser should bundle react app successfully', async ({
  page,
}) => {
  const logs: string[] = [];
  page.on('console', (msg) => {
    logs.push(`[${msg.type()}] ${msg.text()}`);
  });
  page.on('pageerror', (err) =>
    logs.push(`[pageerror] ${err.stack || err.message}`),
  );

  // Track network requests
  const failedRequests: string[] = [];
  page.on('requestfailed', (req) => {
    failedRequests.push(`${req.url()} - ${req.failure()?.errorText}`);
  });

  await page.goto('/basic-react');

  const output = page.locator('#output');
  try {
    await expect(output).toContainText('console.log', { timeout: 90_000 });
  } catch {
    console.log('All browser logs:', logs);
    console.log('Failed requests:', failedRequests);
    // Also check crossOriginIsolated
    const isolated = await page.evaluate(() => window.crossOriginIsolated);
    console.log('crossOriginIsolated:', isolated);
    throw new Error(
      `Test failed.\nBrowser logs:\n${logs.join('\n')}\nFailed requests:\n${failedRequests.join('\n')}`,
    );
  }
  await expect(output).toContainText('rspack');
});
