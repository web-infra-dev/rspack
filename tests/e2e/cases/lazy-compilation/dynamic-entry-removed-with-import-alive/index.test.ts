import { expect, test } from '@/fixtures';

// shared.js is referenced two ways at the same time:
//   - as a dynamic entry (controlled by marker.js)
//   - via import() from main.js
//
// We remove the dynamic entry while the import() is still alive, and
// then prove the proxy stays active by reloading the main page so the
// lazy-compilation client requests `shared` again. Without the
// non-entry-incoming check on `compiler_make`, `shared`'s active state
// would be dropped after the entry removal, breaking subsequent
// import() activations.
test('removing dynamic entry must keep proxy active when import() still references it', async ({
  page,
  fileAction,
  rspack,
}) => {
  // Stage 1: activate via the entry route first.
  await page.goto(
    `http://localhost:${rspack.devServer.options.port}/shared.html`,
  );
  await page.waitForFunction(() => document.body.dataset.shared === '1', null, {
    timeout: 30000,
  });

  // Stage 2: activate via the import() route too.
  await page.goto(`http://localhost:${rspack.devServer.options.port}/`);
  await page.waitForFunction(
    () => document.body.dataset.sharedLoaded === '1',
    null,
    { timeout: 30000 },
  );

  // Stage 3: remove the dynamic-entry marker so the entry function no longer
  // emits `shared`. Touch main.js to make the watcher notice and rerun the
  // entry function (marker.js itself is not in the dep graph).
  fileAction.deleteFile('src/marker.js');
  fileAction.updateFile('src/main.js', (content) => `${content}\n`);
  await rspack.waitingForBuild();
  await new Promise((r) => setTimeout(r, 500));
  await rspack.waitingForBuild();

  const stats = rspack.compiler._lastCompilation
    ?.getStats()
    .toJson({ all: false, errors: true });
  expect(stats?.errors ?? []).toEqual([]);

  // Stage 4: reload main page — the import() must still resolve, proving
  // the proxy stayed active despite the entry removal.
  await page.goto(`http://localhost:${rspack.devServer.options.port}/`);
  await page.waitForFunction(
    () => document.body.dataset.sharedLoaded === '1',
    null,
    { timeout: 30000 },
  );
});
