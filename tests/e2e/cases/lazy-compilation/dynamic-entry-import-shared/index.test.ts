import { expect, test } from '@/fixtures';

// shared.js is referenced two ways at the same time:
//   - via import() from main.js  -> proxy factorized with is_entry = false
//   - as a dynamic entry          -> proxy reused, is_entry stays false
//
// Then we remove the import() first, then remove the entry. Both stages
// must complete without compile errors — the proxy must stay active while
// the import() is gone but entry remains, and must be cleaned up only
// after the entry is also gone.
test('shared file as both import() and entry — remove import() first, then entry', async ({
  page,
  fileAction,
  rspack,
}) => {
  // Stage 1: activate via import(). main.js triggers import('./shared.js'),
  // which factorizes shared as DynamicImport — proxy.is_entry = false.
  await page.goto(`http://localhost:${rspack.devServer.options.port}/`);
  await page.waitForFunction(
    () => document.body.dataset.sharedFromImport === '1',
    null,
    { timeout: 30000 },
  );

  // Stage 2: activate the same proxy via the entry route.
  await page.goto(
    `http://localhost:${rspack.devServer.options.port}/shared.html`,
  );
  await page.waitForFunction(
    () => document.body.dataset.sharedAsEntry === '1',
    null,
    { timeout: 30000 },
  );

  // Stage 3: remove only the import() — entry to shared.js remains.
  // Overwrite main.js entirely; tweaking with regex is brittle because
  // `import().then(...)` spans multiple lines.
  fileAction.updateFile(
    'src/main.js',
    () => "document.body.dataset.main = '1';\n",
  );
  await rspack.waitingForBuild();
  await new Promise((r) => setTimeout(r, 500));
  await rspack.waitingForBuild();

  let stats = rspack.compiler._lastCompilation
    ?.getStats()
    .toJson({ all: false, errors: true });
  expect(stats?.errors ?? []).toEqual([]);

  // Stage 4: delete shared.js — entry to shared.js disappears, watcher
  // reports RemovedFiles. The proxy must be cleaned up cleanly.
  fileAction.deleteFile('src/shared.js');
  await rspack.waitingForBuild();
  await new Promise((r) => setTimeout(r, 500));
  await rspack.waitingForBuild();

  stats = rspack.compiler._lastCompilation
    ?.getStats()
    .toJson({ all: false, errors: true });
  expect(stats?.errors ?? []).toEqual([]);
});
