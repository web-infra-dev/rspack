const fs = __non_webpack_require__('fs');
const path = __non_webpack_require__('path');

__webpack_require__.p = 'PUBLIC_PATH';
it('should not load treeshake shared via set server strategy and no server data dispatch', async () => {
  const app = await import('./App.js');
  expect(app.default()).toEqual(
    'default Uilib has Button, List, Badge exports not treeshake, and ui-lib-es Button value is Button should treeshake',
  );

  const bundlePath = path.join(__dirname, 'node_modules_ui-lib_index_js.js');
  const bundleContent = fs.readFileSync(bundlePath, 'utf-8');
  expect(bundleContent).toContain('Button');
  expect(bundleContent).toContain('Badge');
  expect(bundleContent).toContain('List');

  const uiLibShared =
    __FEDERATION__.__SHARE__['treeshake_share_server:0.0.0'].default['ui-lib'][
      '1.0.0'
    ];
  expect(uiLibShared.loaded).toEqual(true);
  expect(uiLibShared.treeshake.loaded).toEqual(undefined);
  expect(Object.keys(uiLibShared.lib()).sort()).toEqual([
    'Badge',
    'Button',
    'List',
    'default',
  ]);

  const uiLibTreeshake = (await uiLibShared.treeshake.get())();
  expect(Object.keys(uiLibTreeshake).sort()).toEqual(['Button', 'default']);

  const uiLibESBundlePath = path.join(
    __dirname,
    'node_modules_ui-lib-es_index_js.js',
  );
  const uiLibESBundleContent = fs.readFileSync(uiLibESBundlePath, 'utf-8');
  expect(uiLibESBundleContent).toContain('Button');
  expect(uiLibESBundleContent).not.toContain('Badge');
  expect(uiLibESBundleContent).not.toContain('List');

  const uiLibESShared =
    __FEDERATION__.__SHARE__['treeshake_share_server:0.0.0'].default[
      'ui-lib-es'
    ]['1.0.0'];
  expect(uiLibESShared.loaded).toEqual(true);
  expect(uiLibESShared.treeshake.loaded).toEqual(undefined);

  expect(Object.keys(uiLibESShared.lib()).sort()).toEqual([
    'Badge',
    'Button',
    'List',
  ]);

  const esTreeshake = (await uiLibESShared.treeshake.get())();
  expect(Object.keys(esTreeshake).sort()).toEqual(['Button']);
});

it('should treeshake ui-lib-dynamic-specific-export correctly', async () => {
  const { dynamicUISpecificExport } = await import('./App.js');
  expect(await dynamicUISpecificExport()).toEqual(
    'dynamic Uilib has List exports not treeshake',
  );

  const bundlePath = path.join(
    __dirname,
    'node_modules_ui-lib-dynamic-specific-export_index_js.js',
  );
  const bundleContent = fs.readFileSync(bundlePath, 'utf-8');
  expect(bundleContent).toContain('List');
  expect(bundleContent).not.toContain('Button');
  expect(bundleContent).not.toContain('Badge');
});

it('should treeshake ui-lib-dynamic-default-export', async () => {
  const { dynamicUIDefaultExport } = await import('./App.js');
  expect(await dynamicUIDefaultExport()).toEqual(
    'dynamic Uilib has List exports treeshake',
  );

  const bundlePath = path.join(
    __dirname,
    'node_modules_ui-lib-dynamic-default-export_index_js.js',
  );
  const bundleContent = fs.readFileSync(bundlePath, 'utf-8');
  expect(bundleContent).toContain('List');
  expect(bundleContent).toContain('Button');
  expect(bundleContent).toContain('Badge');
});

it('should not treeshake ui-lib-side-effect if not set sideEffect:false ', async () => {
  const { dynamicUISideEffectExport } = await import('./App.js');
  expect(await dynamicUISideEffectExport()).toEqual(
    'dynamic Uilib has List exports not treeshake',
  );

  const bundlePath = path.join(
    __dirname,
    'node_modules_ui-lib-side-effect_index_js.js',
  );
  const bundleContent = fs.readFileSync(bundlePath, 'utf-8');
  expect(bundleContent).toContain('List');
  expect(bundleContent).toContain('Button');
  expect(bundleContent).toContain('Badge');
});

it('should inject usedExports into entry chunk by default', async () => {
  expect(__webpack_require__.federation.usedExports['ui-lib'].sort()).toEqual([
    'Button',
    'default',
  ]);
});

it('should inject usedExports into manifest and stats if enable manifest', async () => {
  const { Button } = await import('ui-lib');
  expect(Button).toEqual('Button');

  const statsPath = path.join(__dirname, 'mf-stats.json');
  const statsContent = JSON.parse(fs.readFileSync(statsPath, 'utf-8'));
  expect(
    JSON.stringify(
      statsContent.shared.find((s) => s.name === 'ui-lib').usedExports.sort(),
    ),
  ).toEqual(JSON.stringify(['Button', 'default']));
});
