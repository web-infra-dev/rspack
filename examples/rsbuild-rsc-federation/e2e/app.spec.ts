import { expect, test } from '@playwright/test';

const appUrls = [
  { name: 'host', url: 'http://localhost:3330/' },
  { name: 'remote-copy', url: 'http://localhost:3331/' },
];

for (const app of appUrls) {
  test(`renders ${app.name} demo app and supports interaction`, async ({
    page,
  }) => {
    const pageErrors: string[] = [];
    page.on('pageerror', (error) => {
      pageErrors.push(error.message);
    });

    await page.goto(app.url);

    await expect(page.getByTestId('app-ready')).toBeVisible();
    await expect(page.getByTestId('status-text')).toHaveText(
      'client entry ready',
    );
    await expect(page.getByTestId('component-rendered')).toHaveText(
      'InteractiveClientDemo',
    );
    await expect(page.getByTestId('counter-value')).toHaveText('0');

    await page.getByTestId('increment-button').click();
    await page.getByTestId('increment-button').click();

    await expect(page.getByTestId('counter-value')).toHaveText('2');
    expect(pageErrors).toEqual([]);
  });

  test(`serves ${app.name} RSC federation artifacts`, async ({ request }) => {
    const baseUrl = app.url.replace(/\/$/, '');
    const [
      statsResponse,
      manifestResponse,
      clientStatsResponse,
      clientManifestResponse,
      remoteEntryResponse,
      remoteEntryClientResponse,
    ] = await Promise.all([
      request.get(`${baseUrl}/mf-stats.json`),
      request.get(`${baseUrl}/mf-manifest.json`),
      request.get(`${baseUrl}/mf-manifest.client-stats.json`),
      request.get(`${baseUrl}/mf-manifest.client.json`),
      request.get(`${baseUrl}/remoteEntry.js`),
      request.get(`${baseUrl}/remoteEntry.client.js`),
    ]);

    expect(statsResponse.status()).toBe(200);
    expect(manifestResponse.status()).toBe(200);
    expect(clientStatsResponse.status()).toBe(200);
    expect(clientManifestResponse.status()).toBe(200);
    expect(remoteEntryResponse.status()).toBe(200);
    expect(remoteEntryClientResponse.status()).toBe(200);

    const stats = await statsResponse.json();
    const manifest = await manifestResponse.json();
    const clientStats = await clientStatsResponse.json();
    const clientManifest = await clientManifestResponse.json();
    const remoteEntryText = await remoteEntryResponse.text();
    const remoteEntryClientText = await remoteEntryClientResponse.text();

    const expectedServerExposes = [
      './button',
      './composed',
      './consumer',
      './server-mixed',
    ];
    const expectedClientExposes = ['./button', './composed'];
    const unexpectedClientExposes = ['./consumer', './server-mixed'];

    const statsExposePaths = stats.exposes.map(
      (expose: { path: string }) => expose.path,
    );
    const manifestExposePaths = manifest.exposes.map(
      (expose: { path: string }) => expose.path,
    );
    const clientStatsExposePaths = clientStats.exposes.map(
      (expose: { path: string }) => expose.path,
    );
    const clientManifestExposePaths = clientManifest.exposes.map(
      (expose: { path: string }) => expose.path,
    );

    for (const exposePath of expectedServerExposes) {
      expect(statsExposePaths).toContain(exposePath);
      expect(manifestExposePaths).toContain(exposePath);
    }

    for (const exposePath of expectedClientExposes) {
      expect(clientStatsExposePaths).toContain(exposePath);
      expect(clientManifestExposePaths).toContain(exposePath);
    }

    for (const exposePath of unexpectedClientExposes) {
      expect(clientStatsExposePaths).not.toContain(exposePath);
      expect(clientManifestExposePaths).not.toContain(exposePath);
    }

    const sharedNames = stats.shared.map(
      (shared: { name: string }) => shared.name,
    );
    expect(sharedNames).toContain('rsbuild-rsc-federation-shared');
    expect(sharedNames).toContain(
      'rsbuild-rsc-federation-shared/server-actions',
    );
    const clientSharedNames = clientStats.shared.map(
      (shared: { name: string }) => shared.name,
    );
    const clientManifestSharedNames = clientManifest.shared.map(
      (shared: { name: string }) => shared.name,
    );
    expect(clientSharedNames).not.toContain(
      'rsbuild-rsc-federation-shared/server-actions',
    );
    expect(clientManifestSharedNames).not.toContain(
      'rsbuild-rsc-federation-shared/server-actions',
    );

    const sharedActionsEntry = stats.shared.find(
      (shared: {
        name: string;
        shareKey?: string;
        rsc?: { serverActions?: Array<{ id: string; name: string }> };
      }) => shared.name === 'rsbuild-rsc-federation-shared/server-actions',
    );
    expect(sharedActionsEntry?.shareKey).toBe('rsc-shared-actions-key');
    expect(sharedActionsEntry?.rsc?.serverActions?.length).toBeGreaterThan(0);

    if (app.name === 'host') {
      expect(stats.name).toBe('rsbuild_host');
      expect(manifest.name).toBe('rsbuild_host');
      expect(clientStats.name).toBe('rsbuild_host');
      expect(clientManifest.name).toBe('rsbuild_host');
      const expectedRemoteModules = ['button', 'consumer', 'server-mixed'];
      for (const moduleName of expectedRemoteModules) {
        const remoteEntry = stats.remotes.find(
          (remote: {
            alias: string;
            moduleName: string;
            rsc?: { lookup?: string };
          }) => remote.alias === 'remote' && remote.moduleName === moduleName,
        );
        expect(remoteEntry?.rsc?.lookup).toBe(`remote/${moduleName}`);
      }
    } else {
      expect(stats.name).toBe('rsbuild_remote');
      expect(manifest.name).toBe('rsbuild_remote');
      expect(clientStats.name).toBe('rsbuild_remote');
      expect(clientManifest.name).toBe('rsbuild_remote');
      expect(stats.remotes).toEqual([]);
      expect(manifest.remotes).toEqual([]);
    }

    expect(remoteEntryText).toContain('server-mixed');
    expect(remoteEntryText).toContain('composed');
    expect(remoteEntryClientText).toContain('composed');
    expect(remoteEntryClientText).toContain('button');
  });
}
