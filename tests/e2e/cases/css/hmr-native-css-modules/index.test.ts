import { test, expect } from '@/fixtures';

const COLOR_A = 'rgb(0, 0, 200)';
const COLOR_B = 'rgb(0, 200, 0)';

// exports stay identical, so the update can only arrive through the
// css manifest entry
test('a style-only edit in a css module applies to the page', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveCSS('color', COLOR_A);
  const className = await root.getAttribute('class');

  fileAction.updateFile('src/style.module.css', (content) =>
    content.replace('rgb(0, 0, 200)', 'rgb(0, 200, 0)'),
  );

  await expect(root).toHaveCSS('color', COLOR_B);
  expect(await root.getAttribute('class')).toBe(className);
});

// changed exports make the css module part of the js update and bubble
// to the entry, which re-applies the new mapping
test('renaming a css module class updates the consumer mapping', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveCSS('color', COLOR_A);
  const before = await root.getAttribute('class');

  fileAction.updateFile('src/style.module.css', (content) =>
    content.replace('.title', '.heading'),
  );
  fileAction.updateFile('src/index.js', (content) =>
    content.replace('styles.title', 'styles.heading'),
  );

  await expect(root).not.toHaveClass(before!);
  await expect(root).toHaveCSS('color', COLOR_A);
});
