const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { NativeWatcher } = require('@rspack/binding');

describe('NativeWatcher raw context events', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-native-raw-'));
  const context = path.join(root, 'src');
  const ignored = path.join(context, 'ignored');
  const existing = path.join(context, 'existing.js');
  const created = path.join(context, 'created.js');
  const ignoredFile = path.join(ignored, 'ignored.js');
  let watcher;

  beforeAll(() => {
    fs.mkdirSync(ignored, { recursive: true });
    fs.writeFileSync(existing, 'export default 1;\n');
  });

  afterAll(async () => {
    await watcher?.close();
    fs.rmSync(root, { recursive: true, force: true });
  });

  it('delivers concrete created and removed children without dropping a raw burst', async () => {
    watcher = new NativeWatcher({
      aggregateTimeout: 10,
      ignored: ['**/ignored/**'],
    });
    const rawEvents = [];
    const aggregated = [];
    const errors = [];

    watcher.watch(
      [[existing], []],
      [[context], []],
      [[], []],
      BigInt(Date.now()),
      (error, result) => {
        if (error) errors.push(error);
        if (result) {
          aggregated.push(result);
          watcher.acknowledgePendingEvents(result.generation);
        }
      },
      (event) => rawEvents.push(event),
    );

    const waitFor = async (predicate) => {
      const deadline = Date.now() + 5000;
      while (!predicate()) {
        if (Date.now() >= deadline) {
          throw new Error(
            `timed out waiting for native raw events: ${JSON.stringify(rawEvents)}`,
          );
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
      }
    };

    fs.writeFileSync(existing, 'export default 2;\n');
    const changedAt = new Date(Date.now() + 2000);
    fs.utimesSync(existing, changedAt, changedAt);
    watcher.triggerEvent('change', existing);
    await waitFor(() => rawEvents.some((event) => event.path === existing));
    rawEvents.length = 0;
    aggregated.length = 0;

    fs.writeFileSync(existing, 'export default 3;\n');
    const nextChangedAt = new Date(Date.now() + 4000);
    fs.utimesSync(existing, nextChangedAt, nextChangedAt);
    fs.writeFileSync(created, 'export default true;\n');
    fs.writeFileSync(ignoredFile, 'export default false;\n');
    watcher.triggerEvent('change', existing);
    watcher.triggerEvent('create', created);
    watcher.triggerEvent('create', ignoredFile);

    await waitFor(
      () =>
        rawEvents.some(
          (event) => event.kind === 'change' && event.path === existing,
        ) &&
        rawEvents.some(
          (event) => event.kind === 'change' && event.path === created,
        ) &&
        aggregated.some(
          (result) =>
            result.changedFiles.includes(context) &&
            result.changedFiles.includes(existing),
        ),
    );
    expect(rawEvents.some((event) => event.path === ignoredFile)).toBe(false);
    expect(
      aggregated.some((result) => result.changedFiles.includes(ignoredFile)),
    ).toBe(false);
    expect(
      aggregated.some((result) => result.changedFiles.includes(created)),
    ).toBe(false);

    const burst = Array.from({ length: 128 }, (_, index) =>
      path.join(context, `created-${index}.js`),
    );
    rawEvents.length = 0;
    aggregated.length = 0;
    for (const file of burst) {
      fs.writeFileSync(file, 'export default true;\n');
      watcher.triggerEvent('create', file);
    }
    await waitFor(
      () =>
        burst.every((file) =>
          rawEvents.some(
            (event) => event.kind === 'change' && event.path === file,
          ),
        ) && aggregated.some((result) => result.changedFiles.includes(context)),
    );
    expect(
      aggregated.some((result) =>
        burst.some((file) => result.changedFiles.includes(file)),
      ),
    ).toBe(false);

    rawEvents.length = 0;
    aggregated.length = 0;
    fs.rmSync(created);
    watcher.triggerEvent('remove', created);
    await waitFor(
      () =>
        rawEvents.some(
          (event) => event.kind === 'remove' && event.path === created,
        ) && aggregated.some((result) => result.changedFiles.includes(context)),
    );

    expect(
      aggregated.some((result) => result.removedFiles.includes(created)),
    ).toBe(false);

    rawEvents.length = 0;
    aggregated.length = 0;
    watcher.triggerEvent('create', created);
    await waitFor(
      () =>
        rawEvents.some(
          (event) => event.kind === 'remove' && event.path === created,
        ) && aggregated.some((result) => result.changedFiles.includes(context)),
    );

    expect(errors).toEqual([]);
  });
});
