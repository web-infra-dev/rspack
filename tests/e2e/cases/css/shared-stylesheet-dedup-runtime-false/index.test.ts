import { test, expect } from '@/fixtures';

test('keeps the loader CSS reload fallback when the extracted runtime is disabled', async ({
  page,
  fileAction,
}) => {
  const links = page.locator('link[rel="stylesheet"]');
  const responses: string[] = [];
  const colors = Array.from(
    { length: 21 },
    (_, index) => `rgb(${10 + index}, ${20 + index}, ${30 + index})`,
  );

  page.on('response', (response) => {
    const url = response.url();
    if (url.includes('/static/style.css')) responses.push(url);
  });

  await expect(page.locator('body')).toHaveCSS('background-color', colors[0]);
  await expect(links).toHaveCount(1);

  for (let index = 1; index < colors.length; index++) {
    const previous = colors[index - 1];
    const next = colors[index];

    fileAction.updateFile('src/index.css', (content) =>
      content.replace(previous, next),
    );
    await expect(page.locator('body')).toHaveCSS('background-color', next);
    await expect(links).toHaveCount(1);
  }

  expect(responses).toHaveLength(colors.length - 1);
  expect(new Set(responses).size).toBe(colors.length - 1);
  expect(responses.every((url) => url.includes('?'))).toBe(true);
});
