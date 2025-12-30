import { test, expect } from '@/fixtures';

test('remove optimized module should not panic', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('#main')).toHaveText('Button');

  fileAction.deleteFile('comp/Button.js');

  const overlay = page.frameLocator('#webpack-dev-server-client-overlay');
  await expect(
    overlay.getByText("Module not found: Can't resolve './Button'"),
  ).toBeVisible();

  fileAction.updateFile(
    'comp/Button.js',
    () => "export const Button = 'NewButton';",
  );
  await expect(page.locator('#main')).toHaveText('NewButton');
});
