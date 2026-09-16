import { expect, test } from '@/fixtures';

async function expect_content(page: any, data: string) {
  await expect
    .poll(async () => {
      await page.reload();
      return await page.locator('div').innerText();
    })
    .toBe(data);
}

test('should compile', async ({ page, fileAction, rspack }) => {
  // rspack.compiler.__sharedObj is injected by plugin in rspack.config.js
  await expect_content(page, '2');

  rspack.compiler.__sharedObj.time++;
  fileAction.updateFile('file.js', (content) => content.replace('1', '2'));

  await expect_content(page, '4');

  rspack.compiler.__sharedObj.time++;
  fileAction.updateFile('file.js', (content) => content.replace('2', '3'));

  await expect_content(page, '6');
});
