import { expect, test } from '@/fixtures';

test('should compile', async ({ page, fileAction, rspack }) => {
  // rspack.compiler.__sharedObj is injected by plugin in rspack.config.js
  await expect(page.locator('#index1')).toHaveText('index1');
  await expect(page.locator('#index2')).toHaveText('index2');

  rspack.compiler.__sharedObj.useFullEntry = false;
  fileAction.updateFile('src/index2.js', (content) =>
    content.replace(
      'div.innerText = "index2";',
      'div.innerText = "index2 updated";',
    ),
  );
  fileAction.updateFile('src/index1.js', (content) =>
    content.replace(
      'div.innerText = "index1";',
      'div.innerText = "index1 updated";',
    ),
  );

  await expect(async () => {
    await page.reload();
    expect(await page.locator('#index1').innerText()).toBe('index1 updated');
  }).toPass();

  await expect(page.locator('#index2')).toHaveCount(0);
  await expect(page.locator('#webpack-dev-server-client-overlay')).toHaveCount(
    0,
  );

  const stats = rspack.compiler._lastCompilation
    ?.getStats()
    .toJson({ all: false, errors: true });
  expect(stats?.errors?.length).toBe(0);
});
