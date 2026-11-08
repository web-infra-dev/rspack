import { expect, test } from '@/fixtures';

// Without incremental make every proxy is re-created each compilation. It used
// to come back inactive, dropping the feature chunk and re-emitting it under
// the same URL on reactivation. The browser caches one ESM namespace per URL,
// so the page stayed on v1.
// https://github.com/web-infra-dev/rspack/issues/15062#issuecomment-5789679637
test('editing an activated lazy esm module applies the new code', async ({
  page,
  fileAction,
}) => {
  await expect(page.locator('body')).toHaveAttribute('data-feature', 'v1', {
    timeout: 30000,
  });

  fileAction.updateFile('src/feature.js', (content) =>
    content.replace("'v1'", "'v2'"),
  );

  await expect(page.locator('body')).toHaveAttribute('data-feature', 'v2', {
    timeout: 30000,
  });
});
