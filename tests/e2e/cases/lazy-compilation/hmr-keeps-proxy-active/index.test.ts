import { expect, test } from '@/fixtures';

// Editing any file used to drop the activated state of every lazy proxy, which
// made the client re-activate them and ask for yet another rebuild.
// https://github.com/web-infra-dev/rspack/issues/15062
test('editing an unrelated file must not rebuild activated lazy proxies', async ({
  page,
  rspack,
  fileAction,
}) => {
  await page.waitForFunction(
    () => document.body.dataset.lazy === 'a,b,c,d',
    null,
    { timeout: 30000 },
  );
  await new Promise((r) => setTimeout(r, 3000));

  let builds = 0;
  rspack.compiler.hooks.done.tap('e2e-build-counter', () => {
    builds++;
  });

  fileAction.updateFile('src/sibling.js', (content) =>
    content.replace('v1', 'v2'),
  );
  await rspack.waitingForBuild();
  await page.waitForFunction(
    () => document.body.dataset.sibling === 'v2',
    null,
    { timeout: 30000 },
  );

  const modules: any[] =
    rspack.compiler._lastCompilation?.getStats().toJson({
      all: false,
      modules: true,
      cachedModules: true,
    }).modules ?? [];
  const proxies = modules.filter((m) =>
    m.identifier?.includes('lazy-compilation-proxy'),
  );

  expect(proxies.length).toBeGreaterThan(0);
  expect(proxies.filter((m) => m.built)).toEqual([]);

  await new Promise((r) => setTimeout(r, 3000));
  expect(builds).toBe(1);
});
