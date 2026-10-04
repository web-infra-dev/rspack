import { test, expect } from '@/fixtures';

type MigrationWindow = Window & {
  sharedInstance: { count: number };
  beforeMigration?: { count: number };
  sharedExecutions: number;
};

export function testInitialChunks() {
  test('loads initial chunks after splitting without repeating an import', async ({
    page,
    context,
    rspack,
    fileAction,
  }) => {
    await expect(page.locator('#root')).toHaveText('shared-v1');
    await page.locator('#state').fill('preserved');
    await page.evaluate(() => {
      (window as MigrationWindow).beforeMigration = (
        window as MigrationWindow
      ).sharedInstance;
      (window as MigrationWindow).sharedInstance.count = 42;
    });
    const requests: string[] = [];
    page.on('request', (request) => requests.push(request.url()));
    const unrelated = await context.newPage();
    await unrelated.goto(new URL('/unrelated.html', page.url()).href);
    await expect(unrelated.locator('#root')).toHaveText('unrelated');
    const unrelatedRequests: string[] = [];
    unrelated.on('request', (request) => unrelatedRequests.push(request.url()));
    const trigger = await context.newPage();
    await trigger.goto(new URL('/trigger.html', page.url()).href);
    await expect
      .poll(() => rspack.devServer.webSocketServer?.clients.length ?? 0)
      .toBeGreaterThanOrEqual(3);
    await trigger.locator('#load').click();
    await expect(trigger.locator('#root')).toHaveText('loaded');

    // This page never invokes import(). HMR must install its new dependency.
    await expect
      .poll(() => requests.some((url) => /shared\..*hot-update/.test(url)), {
        timeout: 10000,
      })
      .toBe(true);
    await expect(page.locator('body')).toHaveCSS(
      'background-color',
      'rgb(21, 22, 23)',
    );
    await expect
      .poll(() =>
        page.evaluate(() => ({
          same:
            (window as MigrationWindow).sharedInstance ===
            (window as MigrationWindow).beforeMigration,
          count: (window as MigrationWindow).sharedInstance.count,
          executions: (window as MigrationWindow).sharedExecutions,
        })),
      )
      .toEqual({ same: true, count: 42, executions: 1 });
    await expect(page.locator('#state')).toHaveValue('preserved', {
      timeout: 10000,
    });
    expect(
      unrelatedRequests.some((url) => /shared.*(?:css|hot-update)/.test(url)),
    ).toBe(false);
    await expect(unrelated.locator('link[rel="stylesheet"]')).toHaveCount(0);

    // A second update must find the newly installed chunk in the HMR state.
    fileAction.updateFile('src/shared.js', (source) =>
      source.replace('shared-v1', 'shared-v2'),
    );
    await expect(page.locator('#root')).toHaveText('shared-v2');
    fileAction.updateFile('src/shared.css', (source) =>
      source.replace('21, 22, 23', '0, 128, 0'),
    );
    await expect(page.locator('body')).toHaveCSS(
      'background-color',
      'rgb(0, 128, 0)',
    );
    await expect(page.locator('#state')).toHaveValue('preserved', {
      timeout: 10000,
    });
  });

  test('installs a new initial factory before executing the updated entry', async ({
    page,
    fileAction,
  }) => {
    await expect(page.locator('#root')).toHaveText('shared-v1');
    await page.locator('#state').fill('preserved');
    fileAction.updateFile(
      'src/app.js',
      (source) =>
        `${source}\nimport { value as added } from './added.js';\ndocument.querySelector('#root').textContent = added;\nimport.meta.webpackHot.accept('./added.js', () => { document.querySelector('#root').textContent = added; });\n`,
    );
    await expect(page.locator('#root')).toHaveText('added-v1', {
      timeout: 10000,
    });
    await expect(page.locator('#state')).toHaveValue('preserved', {
      timeout: 10000,
    });
    fileAction.updateFile('src/added.js', (source) =>
      source.replace('added-v1', 'added-v2'),
    );
    await expect(page.locator('#root')).toHaveText('added-v2');
  });

  test('loads an existing chunk when it becomes an initial dependency', async ({
    page,
    context,
    rspack,
    fileAction,
  }) => {
    await expect(page.locator('#root')).toHaveText('shared-v1');
    await page.locator('#state').fill('preserved');
    const requests: string[] = [];
    page.on('request', (request) => requests.push(request.url()));
    const unrelated = await context.newPage();
    await unrelated.goto(new URL('/unrelated.html', page.url()).href);
    await expect(unrelated.locator('#root')).toHaveText('unrelated');
    await expect
      .poll(() => rspack.devServer.webSocketServer?.clients.length ?? 0)
      .toBeGreaterThanOrEqual(2);
    fileAction.updateFile(
      'src/unrelated.js',
      () => `
      import { value } from './added.js';
      document.querySelector('#root').textContent = value;
      import.meta.webpackHot.accept();
    `,
    );
    await expect(unrelated.locator('#root')).toHaveText('added-v1');
    expect(requests.some((url) => /added\..*hot-update/.test(url))).toBe(false);

    fileAction.updateFile(
      'src/app.js',
      (source) => `${source}
      import { value as added } from './added.js';
      document.querySelector('#root').textContent = added;
    `,
    );
    await expect(page.locator('#root')).toHaveText('added-v1');
    await expect(page.locator('#state')).toHaveValue('preserved');
    expect(requests.some((url) => /added\..*hot-update/.test(url))).toBe(true);
  });

  test('loads a CSS-only initial chunk and follows later CSS and JS updates', async ({
    page,
    fileAction,
  }) => {
    await expect(page.locator('#root')).toHaveText('shared-v1');
    await page.locator('#state').fill('preserved');
    fileAction.updateFile(
      'src/app.js',
      (source) => `${source}\nimport './added.css';\n`,
    );
    await expect(page.locator('body')).toHaveCSS('color', 'rgb(10, 20, 30)');
    await expect(page.locator('#state')).toHaveValue('preserved');
    fileAction.updateFile('src/added.css', (source) =>
      source.replace('10, 20, 30', '40, 50, 60'),
    );
    await expect(page.locator('body')).toHaveCSS('color', 'rgb(40, 50, 60)');
    fileAction.updateFile(
      'src/app.js',
      (source) => `${source}
      import { value as added } from './added.js';
      document.querySelector('#root').textContent = added;
    `,
    );
    await expect(page.locator('#root')).toHaveText('added-v1');
    await expect(page.locator('#state')).toHaveValue('preserved');
  });
}
