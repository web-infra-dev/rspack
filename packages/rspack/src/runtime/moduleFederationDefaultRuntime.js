// @ts-nocheck
var __module_federation_bundler_runtime__,
  __module_federation_runtime_plugins__,
  __module_federation_remote_infos__,
  __module_federation_container_name__,
  __module_federation_share_strategy__,
  __module_federation_share_fallbacks__,
  __module_federation_share_fallback_variants__,
  __module_federation_library_type__;
export default function () {
  const runtimeRequire = __module_federation_runtime_require__;
  if (
    (runtimeRequire.initializeSharingData ||
      runtimeRequire.initializeExposesData) &&
    runtimeRequire.federation
  ) {
    const override = (obj, key, value) => {
      if (!obj) return;
      if (obj[key]) obj[key] = value;
    };
    const merge = (obj, key, fn) => {
      const value = fn();
      if (Array.isArray(value)) {
        obj[key] ??= [];
        obj[key].push(...value);
      } else if (typeof value === 'object' && value !== null) {
        obj[key] ??= {};
        Object.assign(obj[key], value);
      }
    };
    const early = (obj, key, initial) => {
      obj[key] ??= initial();
    };
    const remotesLoadingChunkMapping =
      runtimeRequire.remotesLoadingData?.chunkMapping ?? {};
    const remotesLoadingModuleIdToRemoteDataMapping =
      runtimeRequire.remotesLoadingData?.moduleIdToRemoteDataMapping ?? {};
    const initializeSharingScopeToInitDataMapping =
      runtimeRequire.initializeSharingData?.scopeToSharingDataMapping ?? {};
    const consumesLoadingChunkMapping =
      runtimeRequire.consumesLoadingData?.chunkMapping ?? {};
    const consumesLoadingModuleToConsumeDataMapping =
      runtimeRequire.consumesLoadingData?.moduleIdToConsumeDataMapping ?? {};
    const consumesLoadinginstalledModules = {};
    const initializeSharingInitPromises = [];
    const initializeSharingInitTokens = {};
    const arrayInitializedExternals = new WeakMap();
    const containerShareScope =
      runtimeRequire.initializeExposesData?.shareScope;
    const additionalContainerInitScopes =
      runtimeRequire.initializeSharingData?.additionalInitScopes;

    const createArrayScopeRequire = (shareScopes) => {
      const wrapExternal = (external) => {
        if (!external) return external;
        if (external.then) return external.then(wrapExternal);
        const init = external.init;
        if (typeof init !== 'function') return external;
        const facade = Object.create(external);
        Object.defineProperty(facade, 'init', {
          value: (shareScope, initScope, remoteEntryInitOptions) => {
            let initializedScopes = arrayInitializedExternals.get(external);
            if (!initializedScopes) {
              initializedScopes = new Map();
              arrayInitializedExternals.set(external, initializedScopes);
            }
            const scopesKey = JSON.stringify(shareScopes);
            if (initializedScopes.has(scopesKey)) {
              return initializedScopes.get(scopesKey);
            }
            initializedScopes.set(scopesKey, undefined);
            const result =
              remoteEntryInitOptions === undefined
                ? init.call(
                    external,
                    runtimeRequire.S[shareScopes[0]],
                    initScope,
                  )
                : init.call(
                    external,
                    runtimeRequire.S[shareScopes[0]],
                    initScope,
                    remoteEntryInitOptions,
                  );
            initializedScopes.set(scopesKey, result);
            return result;
          },
        });
        return facade;
      };
      return new Proxy(runtimeRequire, {
        apply(target, thisArg, args) {
          return wrapExternal(Reflect.apply(target, thisArg, args));
        },
      });
    };

    const enableArrayRemoteShareScopes = (instance) => {
      const sharedHandler = instance?.sharedHandler;
      const initializeSharing = sharedHandler?.initializeSharing;
      if (
        typeof initializeSharing !== 'function' ||
        initializeSharing.__rspack_share_scope_array_wrapper__
      ) {
        return;
      }
      const arrayAwareInitializeSharing = function (shareScope, options) {
        const arrayRemotes = [];
        for (const remote of instance.options.remotes) {
          if (
            Array.isArray(remote.shareScope) &&
            remote.shareScope.includes(shareScope)
          ) {
            arrayRemotes.push([remote, remote.shareScope]);
            remote.shareScope = shareScope;
          }
        }
        try {
          return initializeSharing.call(this, shareScope, options);
        } finally {
          for (const [remote, shareScopes] of arrayRemotes) {
            remote.shareScope = shareScopes;
          }
        }
      };
      arrayAwareInitializeSharing.__rspack_share_scope_array_wrapper__ = true;
      sharedHandler.initializeSharing = arrayAwareInitializeSharing;
    };

    for (const key in __module_federation_bundler_runtime__) {
      runtimeRequire.federation[key] =
        __module_federation_bundler_runtime__[key];
    }

    early(
      runtimeRequire.federation,
      'libraryType',
      () => __module_federation_library_type__,
    );
    early(
      runtimeRequire.federation,
      'sharedFallback',
      () => __module_federation_share_fallbacks__,
    );
    const sharedFallback = runtimeRequire.federation.sharedFallback;
    const getSharedFallbackKey = (moduleId, data) => {
      const variants =
        __module_federation_share_fallback_variants__?.[data.shareKey];
      if (!variants) return data.shareKey;
      const expectedScopes = Array.isArray(data.shareScope)
        ? data.shareScope
        : [data.shareScope || 'default'];
      const matchesScope = (variant) => {
        const scopes = Array.isArray(variant.shareScope)
          ? variant.shareScope
          : [variant.shareScope || 'default'];
        return (
          scopes.length === expectedScopes.length &&
          scopes.every((scope, index) => scope === expectedScopes[index])
        );
      };
      const requestMatches = (variant) =>
        !variant.import || variant.import === data.import;
      let matches = variants.filter(
        (variant) =>
          requestMatches(variant) &&
          variant.layer === data.layer &&
          matchesScope(variant),
      );
      if (matches.length === 0 && data.layer !== undefined) {
        matches = variants.filter(
          (variant) =>
            requestMatches(variant) &&
            variant.layer === undefined &&
            matchesScope(variant),
        );
      }
      if (matches.length === 0) return;
      const fallbackKey = `${data.shareKey}\0${moduleId}`;
      sharedFallback[fallbackKey] = matches.map(
        ({ entry, version, globalName }) => [entry, version, globalName],
      );
      return fallbackKey;
    };
    early(
      runtimeRequire.federation,
      'consumesLoadingModuleToHandlerMapping',
      () => {
        const consumesLoadingModuleToHandlerMapping = {};
        for (let [moduleId, data] of Object.entries(
          consumesLoadingModuleToConsumeDataMapping,
        )) {
          const fallbackKey = getSharedFallbackKey(moduleId, data);
          consumesLoadingModuleToHandlerMapping[moduleId] = {
            getter:
              sharedFallback && fallbackKey
                ? runtimeRequire.federation.bundlerRuntime?.getSharedFallbackGetter(
                    {
                      shareKey: fallbackKey,
                      factory: data.fallback,
                      webpackRequire: runtimeRequire,
                      libraryType: runtimeRequire.federation.libraryType,
                    },
                  )
                : data.fallback,
            treeShakingGetter: sharedFallback ? data.fallback : undefined,
            shareInfo: {
              shareConfig: {
                fixedDependencies: false,
                requiredVersion: data.requiredVersion,
                strictVersion: data.strictVersion,
                singleton: data.singleton,
                eager: data.eager,
                layer: data.layer,
              },
              scope: Array.isArray(data.shareScope)
                ? data.shareScope
                : [data.shareScope || 'default'],
            },
            shareKey: data.shareKey,
            treeShaking: runtimeRequire.federation.sharedFallback
              ? {
                  get: data.fallback,
                  mode: data.treeShakingMode,
                }
              : undefined,
          };
        }
        return consumesLoadingModuleToHandlerMapping;
      },
    );
    early(runtimeRequire.federation, 'initOptions', () => ({}));
    early(
      runtimeRequire.federation.initOptions,
      'name',
      () => __module_federation_container_name__,
    );
    early(
      runtimeRequire.federation.initOptions,
      'shareStrategy',
      () => __module_federation_share_strategy__,
    );
    early(runtimeRequire.federation.initOptions, 'shared', () => {
      const shared = {};
      for (let [scope, stages] of Object.entries(
        initializeSharingScopeToInitDataMapping,
      )) {
        for (let stage of stages) {
          if (typeof stage === 'object' && stage !== null) {
            const {
              name,
              version,
              factory,
              eager,
              singleton,
              requiredVersion,
              strictVersion,
              treeShakingMode,
              layer,
            } = stage;
            const shareConfig = {};
            const isValidValue = function (val) {
              return typeof val !== 'undefined';
            };
            if (isValidValue(singleton)) {
              shareConfig.singleton = singleton;
            }
            if (isValidValue(requiredVersion)) {
              shareConfig.requiredVersion = requiredVersion;
            }
            if (isValidValue(eager)) {
              shareConfig.eager = eager;
            }
            if (isValidValue(strictVersion)) {
              shareConfig.strictVersion = strictVersion;
            }
            if (isValidValue(layer)) {
              shareConfig.layer = layer;
            }
            const options = {
              version,
              scope: [scope],
              shareConfig,
              get: factory,
              treeShaking: treeShakingMode
                ? {
                    mode: treeShakingMode,
                  }
                : undefined,
            };
            if (shared[name]) {
              shared[name].push(options);
            } else {
              shared[name] = [options];
            }
          }
        }
      }
      return shared;
    });
    merge(runtimeRequire.federation.initOptions, 'remotes', () =>
      Object.values(__module_federation_remote_infos__)
        .flat()
        .filter((remote) => remote.externalType === 'script'),
    );
    merge(
      runtimeRequire.federation.initOptions,
      'plugins',
      () => __module_federation_runtime_plugins__,
    );

    early(runtimeRequire.federation, 'bundlerRuntimeOptions', () => ({}));
    early(
      runtimeRequire.federation.bundlerRuntimeOptions,
      'remotes',
      () => ({}),
    );
    early(
      runtimeRequire.federation.bundlerRuntimeOptions.remotes,
      'chunkMapping',
      () => remotesLoadingChunkMapping,
    );
    early(
      runtimeRequire.federation.bundlerRuntimeOptions.remotes,
      'remoteInfos',
      () => __module_federation_remote_infos__,
    );
    early(
      runtimeRequire.federation.bundlerRuntimeOptions.remotes,
      'idToExternalAndNameMapping',
      () => {
        const remotesLoadingIdToExternalAndNameMappingMapping = {};
        for (let [moduleId, data] of Object.entries(
          remotesLoadingModuleIdToRemoteDataMapping,
        )) {
          remotesLoadingIdToExternalAndNameMappingMapping[moduleId] = [
            data.shareScope,
            data.name,
            data.externalModuleId,
            data.remoteName,
          ];
        }
        return remotesLoadingIdToExternalAndNameMappingMapping;
      },
    );
    early(
      runtimeRequire.federation.bundlerRuntimeOptions.remotes,
      'webpackRequire',
      () => runtimeRequire,
    );
    merge(
      runtimeRequire.federation.bundlerRuntimeOptions.remotes,
      'idToRemoteMap',
      () => {
        const idToRemoteMap = {};
        for (let [id, remoteData] of Object.entries(
          remotesLoadingModuleIdToRemoteDataMapping,
        )) {
          const existingInfos =
            __module_federation_remote_infos__[remoteData.remoteName] || [];
          const info =
            existingInfos.length > 0
              ? existingInfos
              : remoteData.remoteInfo
                ? [remoteData.remoteInfo]
                : [];
          if (info.length > 0) idToRemoteMap[id] = info;
        }
        return idToRemoteMap;
      },
    );

    override(runtimeRequire, 'S', runtimeRequire.federation.bundlerRuntime.S);
    if (runtimeRequire.federation.attachShareScopeMap) {
      runtimeRequire.federation.attachShareScopeMap(runtimeRequire);
    }

    override(runtimeRequire.f, 'remotes', (chunkId, promises) =>
      runtimeRequire.federation.bundlerRuntime.remotes({
        chunkId,
        promises,
        chunkMapping: remotesLoadingChunkMapping,
        idToExternalAndNameMapping:
          runtimeRequire.federation.bundlerRuntimeOptions.remotes
            .idToExternalAndNameMapping,
        idToRemoteMap:
          runtimeRequire.federation.bundlerRuntimeOptions.remotes.idToRemoteMap,
        webpackRequire: runtimeRequire,
      }),
    );
    const initializeConsumeShareScopes = (moduleIds) => {
      if (!moduleIds?.length) return [];
      const initPromises = [];
      const initializedScopes = new Set();
      for (const moduleId of moduleIds) {
        const shareScope =
          consumesLoadingModuleToConsumeDataMapping[moduleId]?.shareScope ||
          'default';
        const scopeKey = JSON.stringify(
          Array.isArray(shareScope) ? shareScope : [shareScope],
        );
        if (initializedScopes.has(scopeKey)) continue;
        initializedScopes.add(scopeKey);
        const initialized = runtimeRequire.I(shareScope, []);
        if (initialized?.then) initPromises.push(initialized);
      }
      return initPromises;
    };
    override(runtimeRequire.f, 'consumes', (chunkId, promises) => {
      const initialConsumesInit = runtimeRequire.federation.initialConsumesInit;
      if (initialConsumesInit?.then) promises.push(initialConsumesInit);
      const consume = (targetPromises) =>
        runtimeRequire.federation.bundlerRuntime.consumes({
          chunkId,
          promises: targetPromises,
          chunkMapping: consumesLoadingChunkMapping,
          moduleToHandlerMapping:
            runtimeRequire.federation.consumesLoadingModuleToHandlerMapping,
          installedModules: consumesLoadinginstalledModules,
          webpackRequire: runtimeRequire,
        });
      const initPromises = initializeConsumeShareScopes(
        consumesLoadingChunkMapping[chunkId],
      );
      if (initPromises.length === 0) return consume(promises);
      promises.push(
        Promise.all(initPromises).then(() => {
          const consumePromises = [];
          consume(consumePromises);
          return Promise.all(consumePromises);
        }),
      );
    });
    override(runtimeRequire, 'I', (name, initScope) => {
      const webpackRequire = Array.isArray(name)
        ? createArrayScopeRequire(name)
        : runtimeRequire;
      return runtimeRequire.federation.bundlerRuntime.I({
        shareScopeName: name,
        initScope,
        initPromises: initializeSharingInitPromises,
        initTokens: initializeSharingInitTokens,
        webpackRequire,
      });
    });
    override(
      runtimeRequire,
      'initContainer',
      (shareScope, initScope, remoteEntryInitOptions) => {
        let options = remoteEntryInitOptions;
        const additionalScopes = [];
        if (
          additionalContainerInitScopes?.length &&
          options?.shareScopeMap &&
          !Array.isArray(options.shareScopeKeys)
        ) {
          const primaryScope = options.shareScopeKeys || 'default';
          const shareScopeKeys = [primaryScope];
          const containerScopes = Array.isArray(containerShareScope)
            ? containerShareScope
            : [containerShareScope || 'default'];
          for (const scope of additionalContainerInitScopes) {
            if (scope === primaryScope) continue;
            shareScopeKeys.push(scope);
            if (!containerScopes.includes(scope)) additionalScopes.push(scope);
          }
          const descriptors = Object.getOwnPropertyDescriptors(options);
          descriptors.shareScopeKeys = {
            configurable: true,
            enumerable:
              Object.getOwnPropertyDescriptor(options, 'shareScopeKeys')
                ?.enumerable ?? true,
            value: shareScopeKeys,
            writable: true,
          };
          options = Object.create(Object.getPrototypeOf(options), descriptors);
        }
        const result =
          runtimeRequire.federation.bundlerRuntime.initContainerEntry({
            shareScope,
            initScope,
            remoteEntryInitOptions: options,
            shareScopeKey: containerShareScope,
            webpackRequire: runtimeRequire,
          });
        if (additionalScopes.length === 0) return result;
        const initializeAdditionalScopes = () =>
          Promise.all(
            additionalScopes.flatMap((scope) =>
              runtimeRequire.federation.instance.initializeSharing(scope, {
                from: 'build',
                strategy:
                  runtimeRequire.federation.instance.options.shareStrategy,
              }),
            ),
          );
        return result?.then
          ? Promise.resolve(result).then(initializeAdditionalScopes)
          : initializeAdditionalScopes();
      },
    );
    override(runtimeRequire, 'getContainer', (module, getScope) => {
      var moduleMap = runtimeRequire.initializeExposesData.moduleMap;
      runtimeRequire.R = getScope;
      getScope = Object.prototype.hasOwnProperty.call(moduleMap, module)
        ? moduleMap[module]()
        : Promise.resolve().then(() => {
            throw new Error(
              'Module "' + module + '" does not exist in container.',
            );
          });
      runtimeRequire.R = undefined;
      return getScope;
    });

    runtimeRequire.federation.instance =
      runtimeRequire.federation.bundlerRuntime.init({
        webpackRequire: runtimeRequire,
      });
    enableArrayRemoteShareScopes(runtimeRequire.federation.instance);

    if (runtimeRequire.consumesLoadingData?.initialConsumes) {
      const installInitialConsumes = () =>
        runtimeRequire.federation.bundlerRuntime.installInitialConsumes({
          webpackRequire: runtimeRequire,
          installedModules: consumesLoadinginstalledModules,
          initialConsumes: runtimeRequire.consumesLoadingData.initialConsumes,
          moduleToHandlerMapping:
            runtimeRequire.federation.consumesLoadingModuleToHandlerMapping,
        });
      const initPromises = initializeConsumeShareScopes(
        runtimeRequire.consumesLoadingData.initialConsumes,
      );
      if (initPromises.length === 0) {
        installInitialConsumes();
      } else {
        runtimeRequire.federation.initialConsumesInit = Promise.all(
          initPromises,
        ).then(installInitialConsumes);
      }
    }
  }
}
