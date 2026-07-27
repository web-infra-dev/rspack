import { test, expect } from '@/fixtures';

// a js-only edit changes the compilation hash, so the banner rewrites the
// emitted stylesheet during processAssets even though no css module changed;
// diffing the final asset bytes must still list the chunk in the manifest so
// the browser fetches the rewritten stylesheet
test('css rewritten after hashing is delivered on a js-only edit', async ({
  page,
  fileAction,
}) => {
  const root = page.locator('#root');
  await expect(root).toHaveText('step-0');
  const before = await page.evaluate(
    () => getComputedStyle(document.getElementById('root')!, '::after').content,
  );
  expect(before).toContain('"');

  fileAction.updateFile('src/index.js', (content) =>
    content.replace('step-0', 'step-1'),
  );

  await expect(root).toHaveText('step-1');
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          getComputedStyle(document.getElementById('root')!, '::after').content,
      ),
    )
    .not.toBe(before);
});
