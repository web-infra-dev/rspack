const runtime =
  require('../../../packages/rspack/src/runtime/moduleFederationDefaultRuntime.js').default;
const {
  bundlerRuntime,
} = require('@module-federation/runtime-tools/webpack-bundler-runtime');

const magicNames = [
  '__module_federation_bundler_runtime__',
  '__module_federation_runtime_plugins__',
  '__module_federation_remote_infos__',
  '__module_federation_container_name__',
  '__module_federation_share_strategy__',
  '__module_federation_share_fallbacks__',
  '__module_federation_share_fallback_variants__',
  '__module_federation_library_type__',
  '__module_federation_runtime_require__',
];

function createRuntime({
  external,
  externalType = 'commonjs-module',
  remoteShareScope = ['primary', 'secondary'],
  containerShareScope = 'container-custom',
  hasContainer = true,
  sharedFallback,
  sharedFallbackVariants,
  consumeData,
  initialConsumes,
  chunkMapping = {},
  consumeCalls = [],
  additionalInitScopes = [],
  scopeToSharingDataMapping = {},
  // Use the real bundler-runtime initContainerEntry instead of recording
  // its arguments; the runtime under test then gets its own scope map.
  realInitContainerEntry = false,
} = {}) {
  const shareScopeMap = realInitContainerEntry
    ? {}
    : {
        primary: { tag: 'primary' },
        secondary: { tag: 'secondary' },
        'host-custom': { tag: 'host-custom' },
      };
  const runtimeRequire = () => external;
  Object.assign(runtimeRequire, {
    S: shareScopeMap,
    c: {},
    f: {},
    I() {},
    m: {},
    federation: {},
    initializeSharingData: {
      scopeToSharingDataMapping,
      additionalInitScopes,
    },
    remotesLoadingData: {
      chunkMapping: {},
      moduleIdToRemoteDataMapping: {
        remote: {
          shareScope: remoteShareScope,
          name: './module',
          externalModuleId: 'external',
          remoteName: 'remote',
        },
      },
    },
  });
  if (consumeData) {
    runtimeRequire.f.consumes = () => {};
    runtimeRequire.consumesLoadingData = {
      chunkMapping,
      initialConsumes,
      moduleIdToConsumeDataMapping: { consume: consumeData },
    };
  }
  if (hasContainer) {
    runtimeRequire.initContainer = () => {};
    runtimeRequire.initializeExposesData = {
      moduleMap: {},
      shareScope: containerShareScope,
    };
  }

  const matchedScopes = [];
  const initializedScopes = [];
  const initContainerCalls = [];
  const remote = { name: 'remote', shareScope: remoteShareScope };
  const instance = {
    name: 'host',
    options: {
      remotes: [remote],
      shareStrategy: 'version-first',
    },
    shareScopeMap,
    sharedHandler: {
      initializeSharing(shareScope) {
        initializedScopes.push(shareScope);
        if (
          instance.options.remotes.some(
            (item) => item.shareScope === shareScope,
          )
        ) {
          matchedScopes.push(shareScope);
        }
        return [];
      },
    },
    initializeSharing(shareScope, options) {
      return this.sharedHandler.initializeSharing(shareScope, options);
    },
    initOptions() {},
    initShareScopeMap(key, scope) {
      this.shareScopeMap[key] = scope;
    },
    registerShared() {},
  };
  const localBundlerRuntime = {
    ...bundlerRuntime,
    consumes: (options) => {
      consumeCalls.push(options.chunkId);
    },
    init: () => instance,
    getSharedFallbackGetter: ({ shareKey }) => shareKey,
    initContainerEntry: realInitContainerEntry
      ? bundlerRuntime.initContainerEntry
      : (options) => {
          initContainerCalls.push(options);
        },
  };
  const importedBundlerRuntime = {
    ...localBundlerRuntime,
    bundlerRuntime: localBundlerRuntime,
  };
  const remoteInfos = {
    remote: [
      {
        alias: 'remote',
        name: externalType === 'script' ? 'remote' : undefined,
        externalType,
        shareScope: remoteShareScope,
      },
    ],
  };
  const instantiate = new Function(
    ...magicNames,
    `return (${runtime.toString()});`,
  );
  instantiate(
    importedBundlerRuntime,
    [],
    remoteInfos,
    'host',
    'version-first',
    sharedFallback,
    sharedFallbackVariants,
    'commonjs-module',
    runtimeRequire,
  )();

  return {
    initContainerCalls,
    initializedScopes,
    instance,
    matchedScopes,
    remote,
    runtimeRequire,
    shareScopeMap,
  };
}

async function waitForInitialConsume(runtimeRequire) {
  for (let index = 0; index < 10; index++) {
    if (typeof runtimeRequire.m.consume === 'function') return;
    await Promise.resolve();
  }
  expect(runtimeRequire.m.consume).toBeTypeOf('function');
}

const hostShareScopeMap = () => ({
  'host-custom': { tag: 'host-custom' },
  secondary: { tag: 'secondary' },
});

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  {
    description: 'preserves scalar custom-scope container initialization',
    build() {},
    check: () => {
      const { initContainerCalls, instance, runtimeRequire, shareScopeMap } =
        createRuntime({ remoteShareScope: 'host-custom' });
      const remoteEntryInitOptions = { shareScopeKeys: 'host-custom' };
      Object.defineProperty(remoteEntryInitOptions, 'shareScopeMap', {
        value: shareScopeMap,
      });

      runtimeRequire.initContainer(
        shareScopeMap['host-custom'],
        [],
        remoteEntryInitOptions,
      );

      expect(initContainerCalls).toHaveLength(1);
      expect(initContainerCalls[0]).toMatchObject({
        shareScope: shareScopeMap['host-custom'],
        shareScopeKey: 'container-custom',
        remoteEntryInitOptions,
      });
      expect(instance.options.remotes[0].shareScope).toBe('host-custom');
      expect(runtimeRequire.initContainer).toHaveLength(3);
    },
  },
  {
    description: 'expands container init only by declared additional scopes',
    build() {},
    check: async () => {
      const { initContainerCalls, runtimeRequire, shareScopeMap } =
        createRuntime({
          remoteShareScope: 'host-custom',
          additionalInitScopes: ['secondary'],
          scopeToSharingDataMapping: {
            'host-custom': [],
            secondary: [],
          },
        });
      const remoteEntryInitOptions = { shareScopeKeys: 'host-custom' };
      Object.defineProperty(remoteEntryInitOptions, 'shareScopeMap', {
        value: shareScopeMap,
      });

      await runtimeRequire.initContainer(
        shareScopeMap['host-custom'],
        [],
        remoteEntryInitOptions,
      );

      // The host's options reach the bundler runtime untouched (scalar
      // primary binding); additional scopes are mapped separately.
      expect(initContainerCalls).toHaveLength(1);
      expect(initContainerCalls[0].remoteEntryInitOptions).toBe(
        remoteEntryInitOptions,
      );
      expect(remoteEntryInitOptions.shareScopeKeys).toBe('host-custom');
      expect(shareScopeMap.secondary).toBe(
        remoteEntryInitOptions.shareScopeMap.secondary,
      );
    },
  },
  {
    description: 'binds the container primary scope to the host scope object',
    build() {},
    check: async () => {
      const { runtimeRequire, shareScopeMap: containerScopeMap } =
        createRuntime({
          realInitContainerEntry: true,
          containerShareScope: 'container-custom',
          remoteShareScope: 'host-custom',
        });
      const shareScopeMap = hostShareScopeMap();

      await runtimeRequire.initContainer(shareScopeMap['host-custom'], [], {
        shareScopeKeys: 'host-custom',
        shareScopeMap,
      });

      expect(containerScopeMap['container-custom']).toBe(
        shareScopeMap['host-custom'],
      );
    },
  },
  {
    description: 'keeps the primary binding when expanding additional scopes',
    build() {},
    check: async () => {
      const {
        initializedScopes,
        runtimeRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'container-custom',
        remoteShareScope: 'host-custom',
        additionalInitScopes: ['secondary'],
      });
      const shareScopeMap = hostShareScopeMap();

      await runtimeRequire.initContainer(shareScopeMap['host-custom'], [], {
        shareScopeKeys: 'host-custom',
        shareScopeMap,
      });

      // primary alias: container name -> host object
      expect(containerScopeMap['container-custom']).toBe(
        shareScopeMap['host-custom'],
      );
      // additional scope shares the host's object and gets initialized
      expect(containerScopeMap.secondary).toBe(shareScopeMap.secondary);
      expect(initializedScopes).toContain('secondary');
      // the scalar path never registers the host's name on the container
      expect(containerScopeMap['host-custom']).toBeUndefined();
    },
  },
  {
    description:
      'keeps scalar primary binding when the host name is also an additional scope',
    build() {},
    check: async () => {
      const {
        runtimeRequire: containerRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'default',
        additionalInitScopes: ['host-custom'],
      });
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external: { init: containerRequire.initContainer },
        hasContainer: false,
        remoteShareScope: 'host-custom',
      });

      await runtimeRequire.I('host-custom', []);

      expect(containerScopeMap.default).toBe(shareScopeMap['host-custom']);
      expect(containerScopeMap['host-custom']).toBe(
        shareScopeMap['host-custom'],
      );
    },
  },
  {
    description: 'binds a configured array scope by name when initialized alone',
    build() {},
    check: async () => {
      const {
        runtimeRequire: containerRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'default',
        additionalInitScopes: ['primary', 'secondary'],
      });
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external: { init: containerRequire.initContainer },
        hasContainer: false,
        remoteShareScope: ['primary', 'secondary'],
      });

      await runtimeRequire.I('secondary', []);

      expect(containerScopeMap.secondary).toBe(shareScopeMap.secondary);
      expect(containerScopeMap.default).not.toBe(shareScopeMap.secondary);
    },
  },
  {
    description:
      'binds an additional scope by name when the host initializes it, leaving the primary pool alone',
    build() {},
    check: async () => {
      // Hosts initialize a container once per shared scope. Initializing the
      // container's additional `layered-components` scope must bind that
      // scope only; aliasing the primary `default` pool to it mixes scopes.
      const {
        initializedScopes,
        runtimeRequire: containerRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'default',
        remoteShareScope: 'default',
        additionalInitScopes: ['layered-components'],
      });
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external: { init: containerRequire.initContainer },
        hasContainer: false,
        remoteShareScope: 'default',
      });
      shareScopeMap.default = { tag: 'host-default' };
      shareScopeMap['layered-components'] = { tag: 'host-layered' };

      await runtimeRequire.I('layered-components', []);
      expect(containerScopeMap['layered-components']).toBe(
        shareScopeMap['layered-components'],
      );
      expect(containerScopeMap.default).not.toBe(
        shareScopeMap['layered-components'],
      );
      expect(initializedScopes).toContain('layered-components');

      await runtimeRequire.I('default', []);
      expect(containerScopeMap.default).toBe(shareScopeMap.default);
      expect(containerScopeMap['layered-components']).toBe(
        shareScopeMap['layered-components'],
      );
    },
  },
  {
    description:
      'initializes additional providers after binding an array of host scopes',
    build() {},
    check: async () => {
      const {
        initializedScopes,
        runtimeRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'default',
        remoteShareScope: 'default',
        additionalInitScopes: ['default', 'react-layer', 'private'],
      });
      const shareScopeMap = { default: {}, 'react-layer': {}, unrelated: {} };
      containerScopeMap['react-layer'] = { old: true };

      await runtimeRequire.initContainer(shareScopeMap['react-layer'], [], {
        shareScopeKeys: ['react-layer', 'default', 'unrelated'],
        shareScopeMap,
      });

      expect(containerScopeMap.default).toBe(shareScopeMap.default);
      expect(containerScopeMap['react-layer']).toBe(
        shareScopeMap['react-layer'],
      );
      expect(
        initializedScopes.filter((scope) => scope === 'react-layer'),
      ).toHaveLength(1);
      expect(initializedScopes).not.toContain('private');
      expect(initializedScopes).not.toContain('unrelated');
    },
  },
  {
    description: 'does not remap a container-owned scope listed as additional',
    build() {},
    check: async () => {
      // A layered provider makes ShareRuntimeModule list the container's own
      // scope in additionalInitScopes; the host names that scope differently.
      const {
        initializedScopes,
        runtimeRequire,
        shareScopeMap: containerScopeMap,
      } = createRuntime({
        realInitContainerEntry: true,
        containerShareScope: 'container-custom',
        remoteShareScope: 'host-custom',
        additionalInitScopes: ['container-custom', 'secondary'],
      });
      const shareScopeMap = hostShareScopeMap();

      await runtimeRequire.initContainer(shareScopeMap['host-custom'], [], {
        shareScopeKeys: 'host-custom',
        shareScopeMap,
      });

      expect(containerScopeMap['container-custom']).toBe(
        shareScopeMap['host-custom'],
      );
      expect(shareScopeMap['container-custom']).toBeUndefined();
      expect(containerScopeMap.secondary).toBe(shareScopeMap.secondary);
      expect(
        initializedScopes.filter((scope) => scope === 'secondary'),
      ).toHaveLength(1);
    },
  },
  {
    description: 'does not post-initialize scopes owned by the container',
    build() {},
    check: async () => {
      const { initializedScopes, runtimeRequire, shareScopeMap } =
        createRuntime({
          containerShareScope: 'secondary',
          additionalInitScopes: ['secondary'],
        });
      const remoteEntryInitOptions = {
        shareScopeKeys: 'primary',
        shareScopeMap,
      };

      await runtimeRequire.initContainer(
        shareScopeMap.primary,
        [],
        remoteEntryInitOptions,
      );

      expect(initializedScopes).toEqual([]);
    },
  },
  ...['commonjs-module', 'module'].map((externalType) => ({
    description:
      'initializes a legacy %s remote once with the primary array scope'.replace(
        '%s',
        externalType,
      ),
    build() {},
    check: async () => {
      const calls = [];
      const legacyContainer = {
        init(shareScope, initScope) {
          calls.push([shareScope, initScope]);
        },
      };
      const external =
        externalType === 'module'
          ? Promise.resolve(legacyContainer)
          : legacyContainer;
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external,
        externalType,
      });

      await runtimeRequire.I(['primary', 'secondary'], []);

      expect(calls).toHaveLength(1);
      expect(calls[0]).toHaveLength(2);
      expect(calls[0][0]).toBe(shareScopeMap.primary);
    },
  })),
  {
    description:
      'matches every configured script-remote scope during version-first initialization',
    build() {},
    check: async () => {
      const { matchedScopes, remote, runtimeRequire } = createRuntime({
        externalType: 'script',
        hasContainer: false,
      });

      await runtimeRequire.I(['primary', 'secondary'], []);

      expect(matchedScopes).toEqual(['primary', 'secondary']);
      expect(remote.shareScope).toEqual(['primary', 'secondary']);
    },
  },
  {
    description:
      'passes the full ordered scope capability to an enhanced remote once',
    build() {},
    check: async () => {
      const calls = [];
      const enhancedContainer = {
        init(...args) {
          calls.push(args);
        },
      };
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external: enhancedContainer,
      });

      await runtimeRequire.I(['primary', 'secondary'], []);

      expect(calls).toHaveLength(1);
      expect(calls[0]).toHaveLength(3);
      expect(calls[0][0]).toBe(shareScopeMap.primary);
      expect(calls[0][2].shareScopeKeys).toEqual(['primary', 'secondary']);
    },
  },
  {
    description: 'installs scalar initial consumes synchronously',
    build() {},
    check: () => {
      const calls = [];
      const enhancedContainer = {
        init(...args) {
          calls.push(args);
        },
      };
      const { runtimeRequire } = createRuntime({
        external: enhancedContainer,
        remoteShareScope: 'default',
        consumeData: {
          shareKey: 'react',
          shareScope: 'default',
        },
        initialConsumes: ['consume'],
      });

      // Scalar scopes keep the legacy contract: eager factories are available
      // to a synchronous entry, installation is never gated, and the scope is
      // initialized lazily by the consume handlers (initializing it here would
      // start loading eager shares asynchronously).
      expect(runtimeRequire.m.consume).toBeTypeOf('function');
      expect(runtimeRequire.federation.initialConsumesInit).toBeUndefined();
      expect(calls).toHaveLength(0);
    },
  },
  {
    description:
      'waits for a pending scalar scope initialization before consuming from a chunk',
    build() {},
    check: async () => {
      // A `module`/`promise` remote registers its shares asynchronously; a
      // chunk's consumes must not resolve to the local fallback before that.
      let resolveExternal;
      const external = new Promise((resolve) => {
        resolveExternal = resolve;
      });
      const consumeCalls = [];
      const { runtimeRequire } = createRuntime({
        external,
        remoteShareScope: 'default',
        consumeData: {
          shareKey: 'react',
          shareScope: 'default',
        },
        chunkMapping: { chunk: ['consume'] },
        consumeCalls,
      });

      const promises = [];
      runtimeRequire.f.consumes('chunk', promises);
      expect(promises).toHaveLength(1);
      expect(consumeCalls).toEqual([]);
      resolveExternal({ init() {} });
      await Promise.all(promises);
      expect(consumeCalls).toEqual(['chunk']);
    },
  },
  {
    description:
      'initializes ordered scopes before installing initial consumes',
    build() {},
    check: async () => {
      const calls = [];
      const enhancedContainer = {
        init(...args) {
          calls.push(args);
        },
      };
      const { runtimeRequire, shareScopeMap } = createRuntime({
        external: enhancedContainer,
        consumeData: {
          shareKey: 'react',
          shareScope: ['primary', 'secondary'],
        },
        initialConsumes: ['consume'],
      });

      expect(calls).toHaveLength(1);
      expect(calls[0][0]).toBe(shareScopeMap.primary);
      expect(calls[0][2].shareScopeKeys).toEqual(['primary', 'secondary']);
      await waitForInitialConsume(runtimeRequire);
    },
  },
  {
    description: 'waits for ordered scopes before installing initial consumes',
    build() {},
    check: async () => {
      let resolveInit;
      const initPromise = new Promise((resolve) => {
        resolveInit = resolve;
      });
      const enhancedContainer = {
        init() {
          return initPromise;
        },
      };
      const { runtimeRequire } = createRuntime({
        external: enhancedContainer,
        consumeData: {
          shareKey: 'react',
          shareScope: ['primary', 'secondary'],
        },
        initialConsumes: ['consume'],
      });

      expect(runtimeRequire.m.consume).toBeUndefined();
      const startupPromises = [];
      runtimeRequire.f.consumes('startup', startupPromises);
      expect(startupPromises).toContain(
        runtimeRequire.federation.initialConsumesInit,
      );
      resolveInit();
      await waitForInitialConsume(runtimeRequire);
    },
  },
  {
    description:
      'initializes a frozen legacy container without proxying its exports',
    build() {},
    check: async () => {
      const calls = [];
      const external = Object.freeze({
        init(...args) {
          calls.push(args);
        },
      });
      const { runtimeRequire, shareScopeMap } = createRuntime({ external });

      await runtimeRequire.I(['primary', 'secondary'], []);

      expect(calls).toHaveLength(1);
      expect(calls[0][0]).toBe(shareScopeMap.primary);
    },
  },
  {
    description:
      'does not retry a legacy container whose init throws synchronously',
    build() {},
    check: async () => {
      let calls = 0;
      const external = {
        init() {
          calls += 1;
          throw new Error('legacy init failed');
        },
      };
      const { runtimeRequire } = createRuntime({ external });

      await runtimeRequire.I(['primary', 'secondary'], []);

      expect(calls).toBe(1);
    },
  },
  {
    description: 'selects tree-shaking fallbacks by scope and layer identity',
    build() {},
    check: () => {
      const sharedFallback = {
        react: [
          ['legacy.js', '1.0.0', 'legacy'],
          ['server.js', '1.0.0', 'server'],
        ],
      };
      const { runtimeRequire } = createRuntime({
        sharedFallback,
        sharedFallbackVariants: {
          react: [
            {
              entry: 'legacy.js',
              version: '1.0.0',
              globalName: 'legacy',
              shareScope: 'primary',
              import: 'react',
            },
            {
              entry: 'server.js',
              version: '1.0.0',
              globalName: 'server',
              shareScope: 'primary',
              layer: 'server',
              import: 'react-server',
            },
          ],
        },
        consumeData: {
          shareKey: 'react',
          shareScope: 'primary',
          layer: 'server',
          import: 'react-server',
        },
      });

      const fallbackKey =
        runtimeRequire.federation.consumesLoadingModuleToHandlerMapping.consume
          .getter;
      expect(fallbackKey).toBe('react\0consume');
      expect(runtimeRequire.federation.sharedFallback[fallbackKey]).toEqual([
        ['server.js', '1.0.0', 'server'],
      ]);
      expect(sharedFallback.react).toHaveLength(2);
    },
  },
  {
    description:
      'uses the consume fallback when no tree-shaking variant matches',
    build() {},
    check: () => {
      const fallback = () => 'local';
      const { runtimeRequire } = createRuntime({
        sharedFallback: { react: [['client.js', '1.0.0', 'client']] },
        sharedFallbackVariants: {
          react: [
            {
              entry: 'client.js',
              version: '1.0.0',
              globalName: 'client',
              shareScope: 'client',
              import: 'react-client',
            },
          ],
        },
        consumeData: {
          shareKey: 'react',
          shareScope: 'server',
          import: 'react-server',
          fallback,
        },
      });

      expect(
        runtimeRequire.federation.consumesLoadingModuleToHandlerMapping.consume
          .getter,
      ).toBe(fallback);
    },
  },
];
