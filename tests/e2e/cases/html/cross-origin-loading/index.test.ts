import { test, expect } from '@/fixtures';

test('should set crossOrigin to anonymous for script tag correctly', async ({
  page,
}) => {
  const scripts = await page.$$('script');

  const crossOrigins = await Promise.all(
    scripts.map((script) => script.getAttribute('crossorigin')),
  );

  const srcPaths = await Promise.all(
    scripts.map((script) => script.getAttribute('src')),
  );

  expect(srcPaths).toEqual([
    'main.js',
    expect.stringMatching(
      /^https:\/\/cdn\.example\.com\/src_foo_js\.js\?hmr=[a-f0-9]+$/,
    ),
  ]);
  expect(crossOrigins).toEqual([null, 'anonymous']);
});
