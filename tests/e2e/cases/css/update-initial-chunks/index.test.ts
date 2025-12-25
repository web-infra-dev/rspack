import { test, expect } from '@/fixtures';

test('should update body css', async ({ page, fileAction }) => {
  fileAction.updateFile('src/index.js', (content) => content.replace('//', ''));

  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    'rgb(163, 255, 255)',
  );
});
