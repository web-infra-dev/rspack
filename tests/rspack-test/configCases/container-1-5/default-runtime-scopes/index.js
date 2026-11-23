const runtime = require('default-runtime').default;
const { bundlerRuntime } = require('bundler-runtime');

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

function createRuntime({ consume } = {}) {
  const containerScopeMap = {};
  const runtimeRequire = Object.assign(() => {}, {
    S: containerScopeMap,
    c: {},
    f: {},
    I() {},
    m: {},
    federation: {},
    initializeSharingData: { scopeToSharingDataMapping: {} },
    initializeExposesData: { moduleMap: {}, shareScope: 'container-custom' },
    initContainer() {},
  });
  if (consume) {
    runtimeRequire.consumesLoadingData = {
      initialConsumes: ['consume'],
      moduleIdToConsumeDataMapping: {
        consume: { shareKey: 'react', shareScope: 'default' },
      },
    };
  }

  const importedBundlerRuntime = {
    bundlerRuntime: {
      ...bundlerRuntime,
      S: containerScopeMap,

      init: () => ({
        options: { shareStrategy: 'version-first' },

        initOptions() {},
        initShareScopeMap(key, scope) {
          containerScopeMap[key] = scope;
        },
        initializeSharing() {
          return [];
        },
        registerShared() {},
        loadShareSync: () => () => consume,
      }),
    },
  };
  const instantiate = new Function(
    ...magicNames,
    `return (${runtime.toString()});`,
  );
  instantiate(
    importedBundlerRuntime,
    [],
    {},
    'host',
    'version-first',
    undefined,
    undefined,
    'commonjs-module',
    runtimeRequire,
  )();
  return { runtimeRequire, containerScopeMap };
}

it('binds the container primary scope to the host scope object', async () => {
  const { runtimeRequire, containerScopeMap } = createRuntime();
  const hostScope = { tag: 'host-custom' };

  await runtimeRequire.initContainer(hostScope, [], {
    shareScopeKeys: 'host-custom',
    shareScopeMap: { 'host-custom': hostScope },
  });

  expect(containerScopeMap['container-custom']).toBe(hostScope);
});

it('installs scalar initial consumes synchronously', () => {
  const consume = { name: 'react' };
  const { runtimeRequire } = createRuntime({ consume });
  const module = { exports: {} };

  runtimeRequire.m.consume(module);

  expect(module.exports).toBe(consume);
});
