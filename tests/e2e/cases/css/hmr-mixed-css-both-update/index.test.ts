import { test, expect } from '@/fixtures';

// one rebuild changes an extracted stylesheet and a native css stylesheet
// at once: the css and miniCss manifest namespaces are both active and each
// runtime applies only its own update
test('extract and native stylesheets update together in one hot update', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  const body = page.locator('body');
  await expect(root).toHaveCSS('color', 'rgb(200, 0, 0)');
  await expect(body).toHaveCSS('background-color', 'rgb(163, 255, 255)');

  fileAction.updateFile('src/extract.css', (content) =>
    content.replace('rgb(200, 0, 0)', 'rgb(0, 0, 200)'),
  );
  fileAction.updateFile('src/native.ncss', (content) =>
    content.replace('rgb(163, 255, 255)', 'rgb(0, 128, 0)'),
  );

  await expect(root).toHaveCSS('color', 'rgb(0, 0, 200)');
  await expect(body).toHaveCSS('background-color', 'rgb(0, 128, 0)');
});
