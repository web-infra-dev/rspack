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

    const createShareScopeRequire = (shareScopes) => {
      const ordered = Array.isArray(shareScopes);
      const wrapExternal = (external, externalModuleId) => {
        if (!ordered) {
          const remote = Object.values(
            remotesLoadingModuleIdToRemoteDataMapping,
          ).find((remote) => remote.externalModuleId === externalModuleId);
          if (
            !remote ||
            (!Array.isArray(remote.shareScope) &&
              (remote.shareScope || 'default') === shareScopes)
          ) {
            return external;
          }
        }
        if (!external) return external;
        if (external.then) {
          return external.then((external) =>
            wrapExternal(external, externalModuleId),
          );
        }
        const init = external.init;
        if (typeof init !== 'function') return external;
        const facade = Object.create(external);
        Object.defineProperty(facade, 'init', {
          value: (shareScope, initScope, remoteEntryInitOptions) => {
            if (!ordered) {
              return init.call(
                external,
                shareScope,
                initScope,
                remoteEntryInitOptions === undefined
                  ? undefined
                  : withShareScopeKeys(remoteEntryInitOptions, [shareScopes]),
              );
            }
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
          return wrapExternal(Reflect.apply(target, thisArg, args), args[0]);
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
        (!variant.import || variant.import === data.import) &&
        (!variant.resource || variant.resource === data.resource);
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
              : remoteData.remoteInfos || [];
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
    // Initializes the share scopes of the given consume modules and returns
    // the promises to wait for before consuming.
    // - Chunk path (`includeScalar`): version-first scopes are initialized up front and
    //   awaited, so a remote registered through `I()` (a `module`/`promise`
    //   external initializes asynchronously) contributes its shares before a
    //   consume resolves to a local fallback. Chunk loading is asynchronous
    //   anyway. Loaded-first scalar consumes keep remote loading lazy.
    // - Initial path: only ordered (array) scopes, which the bundler runtime
    //   cannot initialize lazily; they enable async startup, which awaits
    //   `initialConsumesInit`. Scalar scopes are left to the consume handlers:
    //   initializing them here would start loading eager shares
    //   asynchronously and make a synchronous eager consume fail, and
    //   `initializeSharing` always returns a promise, so installation could
    //   not stay synchronous either.
    const initializeConsumeShareScopes = (moduleIds, includeScalar) => {
      if (!moduleIds?.length) return [];
      const initPromises = [];
      const initializedScopes = new Set();
      for (const moduleId of moduleIds) {
        const shareScope =
          consumesLoadingModuleToConsumeDataMapping[moduleId]?.shareScope ||
          'default';
        const ordered = Array.isArray(shareScope);
        if (!ordered && !includeScalar) continue;
        const scopeKey = JSON.stringify(ordered ? shareScope : [shareScope]);
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
        runtimeRequire.federation.instance.options.shareStrategy !==
          'loaded-first',
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
      return runtimeRequire.federation.bundlerRuntime.I({
        shareScopeName: name,
        initScope,
        initPromises: initializeSharingInitPromises,
        initTokens: initializeSharingInitTokens,
        webpackRequire: createShareScopeRequire(name),
      });
    });
    // Returns `options` with `shareScopeKeys` replaced, preserving the
    // prototype and property descriptors of the host's object.
    const withShareScopeKeys = (options, shareScopeKeys) => {
      const descriptors = Object.getOwnPropertyDescriptors(options);
      descriptors.shareScopeKeys = {
        configurable: true,
        enumerable:
          Object.getOwnPropertyDescriptor(options, 'shareScopeKeys')
            ?.enumerable ?? true,
        value: shareScopeKeys,
        writable: true,
      };
      return Object.create(Object.getPrototypeOf(options), descriptors);
    };
    override(
      runtimeRequire,
      'initContainer',
      (shareScope, initScope, remoteEntryInitOptions) => {
        // Scalar initialization binds the container's primary scope to the
        // supplied host object, even when their names differ. The host tags
        // explicit additional-scope calls with array keys to bind by name.
        // Scopes the container owns are bound by initContainerEntry itself;
        // map and initialize its remaining additional scopes here.
        const hostShareScopeMap = remoteEntryInitOptions?.shareScopeMap;
        const additionalScopes = [];
        if (additionalContainerInitScopes?.length && hostShareScopeMap) {
          const hostScope = remoteEntryInitOptions.shareScopeKeys || 'default';
          const containerScopes = Array.isArray(containerShareScope)
            ? containerShareScope
            : [containerShareScope || 'default'];
          if (Array.isArray(hostScope)) {
            // The bundler binds every host scope, but initializes only the
            // container's primary scopes. Register additional providers too.
            additionalScopes.push(
              ...hostScope.filter(
                (scope) =>
                  additionalContainerInitScopes.includes(scope) &&
                  !containerScopes.includes(scope),
              ),
            );
          } else {
            for (const scope of additionalContainerInitScopes) {
              if (containerScopes.includes(scope)) continue;
              if (!hostShareScopeMap[scope]) hostShareScopeMap[scope] = {};
              runtimeRequire.federation.instance.initShareScopeMap(
                scope,
                hostShareScopeMap[scope],
                { hostShareScopeMap },
              );
              additionalScopes.push(scope);
            }
          }
        }
        const result =
          runtimeRequire.federation.bundlerRuntime.initContainerEntry({
            shareScope,
            initScope,
            remoteEntryInitOptions,
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
        false,
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
