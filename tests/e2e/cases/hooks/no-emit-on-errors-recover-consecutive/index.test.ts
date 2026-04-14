import { expect, test } from '@/fixtures';

test('should recover after consecutive errors with emitOnErrors: false', async ({
  page,
  fileAction,
  rspack,
}) => {
  // Step 0: initial build succeeds
  await expect(page.getByText('value:1')).toBeVisible();

  // Step 1: introduce syntax error
  fileAction.updateFile('src/index.js', () => ']});\nexport default 2;');
  await rspack.waitingForBuild();
  await expect(page.locator('#rspack-dev-server-client-overlay')).toHaveCount(
    1,
  );
  await expect(page.getByText('value:1')).toBeVisible();

  // Step 2: introduce a different syntax error
  fileAction.updateFile('src/index.js', () => 'export default <<<<<<;');
  await rspack.waitingForBuild();
  await expect(page.locator('#rspack-dev-server-client-overlay')).toHaveCount(
    1,
  );
  await expect(page.getByText('value:1')).toBeVisible();

  // Step 3: fix the error → HMR should compare against step 0 (last good compilation)
  fileAction.updateFile('src/index.js', () =>
    [
      'const div = document.getElementById("root") || document.createElement("div");',
      'div.id = "root";',
      'div.innerText = "value:4";',
      'document.body.appendChild(div);',
      'if (module.hot) { module.hot.accept(); }',
    ].join('\n'),
  );
  await rspack.waitingForBuild();
  await expect(page.locator('#rspack-dev-server-client-overlay')).toHaveCount(
    0,
  );
  await expect(page.getByText('value:4')).toBeVisible();
});
