import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';

const pluginPath = path.resolve(
  import.meta.dirname,
  '../src/framework/mf-rsc-registration-runtime-plugin.js',
);

function createManifest({
  buildVersion,
  clientReferences,
  serverAction,
  sharedClientReferences,
  sharedServerAction,
}) {
  return {
    id: 'rsbuild_remote',
    name: 'rsbuild_remote',
    metaData: {
      buildInfo: {
        buildVersion,
      },
    },
    exposes: [
      {
        path: './button',
        name: 'button',
        rsc: {
          lookup: 'remote/button',
          resource: 'remote/button',
          clientReferences,
          serverActions: [serverAction],
        },
      },
    ],
    shared: [
      {
        name: 'rsbuild-rsc-federation-shared',
        shareKey: 'rsc-shared-key',
        rsc: {
          lookup: 'rsc-shared-key',
          resource: 'rsbuild-rsc-federation-shared',
          clientReferences: sharedClientReferences,
          serverActions: [sharedServerAction],
        },
      },
    ],
  };
}

function createRemoteArgs(origin) {
  return {
    origin,
    remote: {
      alias: 'remote',
      name: 'rsbuild_remote',
      entry: 'http://localhost:3331/mf-manifest.json',
    },
    remoteInfo: {
      alias: 'remote',
      name: 'rsbuild_remote',
      entry: 'http://localhost:3331/mf-manifest.json',
      buildVersion: 'local',
    },
  };
}

function executeFactory(webpackRequire, moduleId) {
  const module = { exports: {} };
  webpackRequire.m[moduleId](module);
  return module.exports;
}

function toPlainJson(value) {
  return JSON.parse(JSON.stringify(value));
}

const runtimePluginSource = await fs.readFile(pluginPath, 'utf8');
const loadRemoteCalls = [];
const loadShareCalls = [];
let remoteModuleExports = {
  default: 'remote-default',
  namedClient: 'named-client',
  actionAlpha: async () => 'action-alpha',
};
let sharedModuleExports = {
  SharedClientComponent: 'shared-client',
  sharedActionAlpha: async () => 'shared-action-alpha',
};

const webpackRequire = {
  m: {},
  c: {},
  rscM: undefined,
  federation: {
    instance: {
      loadRemote: async (request) => {
        loadRemoteCalls.push(request);
        return remoteModuleExports;
      },
      loadShare: async (pkgName) => {
        loadShareCalls.push(pkgName);
        return () => sharedModuleExports;
      },
    },
  },
};

const context = vm.createContext({
  console,
  Response,
  fetch: async () => {
    throw new Error('unexpected fetch');
  },
  __webpack_require__: webpackRequire,
});

const module = new vm.SourceTextModule(runtimePluginSource, {
  context,
  identifier: pluginPath,
});
await module.link(() => {
  throw new Error('unexpected import');
});
await module.evaluate();

const createPlugin = module.namespace.default;
const plugin = createPlugin();
const manifestCache = new Map();
const origin = {
  snapshotHandler: {
    manifestCache,
  },
};

const firstManifest = createManifest({
  buildVersion: 'local',
  clientReferences: ['default'],
  serverAction: { id: 'remote-action-alpha', name: 'actionAlpha' },
  sharedClientReferences: ['SharedClientComponent'],
  sharedServerAction: {
    id: 'shared-action-alpha',
    name: 'sharedActionAlpha',
  },
});

await plugin.loadRemoteSnapshot({
  from: 'manifest',
  manifestJson: firstManifest,
  moduleInfo: {
    alias: 'remote',
    name: 'rsbuild_remote',
    entry: 'http://localhost:3331/mf-manifest.json',
  },
});
await plugin.afterResolve(createRemoteArgs(origin));

assert.equal(
  webpackRequire.rscM.clientManifest['remote/button'].name,
  '*',
  'expected remote client manifest entry',
);
assert.equal(
  webpackRequire.rscM.clientManifest['rsc-shared-key'].name,
  '*',
  'expected shared client manifest entry',
);
assert.ok(
  webpackRequire.rscM.serverManifest['remote-action-alpha'],
  'expected remote server action entry',
);
assert.ok(
  webpackRequire.rscM.serverManifest['shared-action-alpha'],
  'expected shared server action entry',
);

const remoteClientModuleId =
  webpackRequire.rscM.clientManifest['remote/button'].id;
const sharedClientModuleId =
  webpackRequire.rscM.clientManifest['rsc-shared-key'].id;
const remoteActionModuleId =
  webpackRequire.rscM.serverManifest['remote-action-alpha'].id;
const sharedActionModuleId =
  webpackRequire.rscM.serverManifest['shared-action-alpha'].id;

const firstRemoteClientModule = await executeFactory(
  webpackRequire,
  remoteClientModuleId,
);
assert.deepEqual(toPlainJson(firstRemoteClientModule), {
  default: 'remote-default',
});

const firstSharedClientModule = await executeFactory(
  webpackRequire,
  sharedClientModuleId,
);
assert.deepEqual(toPlainJson(firstSharedClientModule), {
  SharedClientComponent: 'shared-client',
});

const firstRemoteActionModule = executeFactory(
  webpackRequire,
  remoteActionModuleId,
);
assert.equal(
  await firstRemoteActionModule['remote-action-alpha'](),
  'action-alpha',
);

const firstSharedActionModule = executeFactory(
  webpackRequire,
  sharedActionModuleId,
);
assert.equal(
  await firstSharedActionModule['shared-action-alpha'](),
  'shared-action-alpha',
);

remoteModuleExports = {
  default: 'remote-default',
  namedClient: 'named-client-updated',
  actionBeta: async () => 'action-beta',
};
sharedModuleExports = {
  SharedClientComponent: 'shared-client-updated',
  sharedActionBeta: async () => 'shared-action-beta',
};

const secondManifest = createManifest({
  buildVersion: 'local',
  clientReferences: ['namedClient'],
  serverAction: { id: 'remote-action-beta', name: 'actionBeta' },
  sharedClientReferences: ['SharedClientComponent'],
  sharedServerAction: {
    id: 'shared-action-beta',
    name: 'sharedActionBeta',
  },
});

await plugin.loadRemoteSnapshot({
  from: 'manifest',
  manifestJson: secondManifest,
  moduleInfo: {
    alias: 'remote',
    name: 'rsbuild_remote',
    entry: 'http://localhost:3331/mf-manifest.json',
  },
});
await plugin.afterResolve(createRemoteArgs(origin));

assert.equal(
  webpackRequire.rscM.serverManifest['remote-action-alpha'],
  undefined,
  'expected stale remote action entry to be removed',
);
assert.equal(
  webpackRequire.rscM.serverManifest['shared-action-alpha'],
  undefined,
  'expected stale shared action entry to be removed',
);
assert.ok(
  webpackRequire.rscM.serverManifest['remote-action-beta'],
  'expected updated remote action entry',
);
assert.ok(
  webpackRequire.rscM.serverManifest['shared-action-beta'],
  'expected updated shared action entry',
);

const updatedRemoteClientModule = await executeFactory(
  webpackRequire,
  webpackRequire.rscM.clientManifest['remote/button'].id,
);
assert.deepEqual(toPlainJson(updatedRemoteClientModule), {
  namedClient: 'named-client-updated',
});

const updatedRemoteActionModule = executeFactory(
  webpackRequire,
  webpackRequire.rscM.serverManifest['remote-action-beta'].id,
);
assert.equal(
  await updatedRemoteActionModule['remote-action-beta'](),
  'action-beta',
);

assert.ok(loadRemoteCalls.length >= 2, 'expected remote module loads');
assert.ok(
  loadRemoteCalls.every((request) => request === 'remote/button'),
  'expected remote loads to target remote/button',
);
assert.ok(loadShareCalls.length >= 2, 'expected shared module loads');
assert.ok(
  loadShareCalls.every(
    (pkgName) => pkgName === 'rsbuild-rsc-federation-shared',
  ),
  'expected shared loads to target rsbuild-rsc-federation-shared',
);

console.log('[verify-runtime-plugin] runtime plugin registration verified');
