// @ts-nocheck
var __module_federation_bundler_runtime__,
  __module_federation_runtime_plugins__,
  __module_federation_remote_infos__,
  __module_federation_container_name__,
  __module_federation_share_strategy__,
  __module_federation_share_fallbacks__,
  __module_federation_library_type__;
export default function () {
  if (
    (__webpack_require__.initializeSharingData ||
      __webpack_require__.initializeExposesData) &&
    __webpack_require__.federation
  ) {
    const toScopeArray = (scope) => {
      if (Array.isArray(scope)) {
        const normalized = scope.filter(Boolean);
        return normalized.length > 0 ? normalized : ['default'];
      }
      if (typeof scope === 'string' && scope) {
        return [scope];
      }
      return ['default'];
    };
    const toRuntimeScope = (scope) => {
      const scopes = toScopeArray(scope);
      return scopes.length === 1 ? scopes[0] : scopes;
    };
    const normalizeBundlerRemoteInfo = (remote) => {
      // Keep runtime remotes with concrete entry/name info intact.
      // For build-time remotes (no name/entry), align with enhanced output so
      // sharing init still initializes externals before consume handlers run.
      if (remote && (remote.name || remote.entry)) {
        return remote;
      }
      return Object.assign({}, remote, {
        name: '',
        entry: '',
        externalType: 'unknown',
      });
    };
    const bundlerRuntimeRemoteInfos = Object.fromEntries(
      Object.entries(__module_federation_remote_infos__).map(([key, infos]) => [
        key,
        infos.map(normalizeBundlerRemoteInfo),
      ]),
    );
    const expandRemoteScopes = (remote) =>
      toScopeArray(remote.shareScope).map((scope) => {
        const normalized = Object.assign({}, remote);
        normalized.shareScope = scope;
        return normalized;
      });

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
      __webpack_require__.remotesLoadingData?.chunkMapping ?? {};
    const remotesLoadingModuleIdToRemoteDataMapping =
      __webpack_require__.remotesLoadingData?.moduleIdToRemoteDataMapping ?? {};
    const initializeSharingScopeToInitDataMapping =
      __webpack_require__.initializeSharingData?.scopeToSharingDataMapping ??
      {};
    const consumesLoadingChunkMapping =
      __webpack_require__.consumesLoadingData?.chunkMapping ?? {};
    const consumesLoadingModuleToConsumeDataMapping =
      __webpack_require__.consumesLoadingData?.moduleIdToConsumeDataMapping ??
      {};
    const consumesLoadinginstalledModules = {};
    const initializeSharingInitPromises = [];
    const initializeSharingInitTokens = {};
    const containerShareScope = toRuntimeScope(
      __webpack_require__.initializeExposesData?.shareScope,
    );

    for (const key in __module_federation_bundler_runtime__) {
      __webpack_require__.federation[key] =
        __module_federation_bundler_runtime__[key];
    }

    early(
      __webpack_require__.federation,
      'libraryType',
      () => __module_federation_library_type__,
    );
    early(
      __webpack_require__.federation,
      'sharedFallback',
      () => __module_federation_share_fallbacks__,
    );
    const sharedFallback = __webpack_require__.federation.sharedFallback;
    early(
      __webpack_require__.federation,
      'consumesLoadingModuleToHandlerMapping',
      () => {
        const consumesLoadingModuleToHandlerMapping = {};
        for (let [moduleId, data] of Object.entries(
          consumesLoadingModuleToConsumeDataMapping,
        )) {
          consumesLoadingModuleToHandlerMapping[moduleId] = {
            getter: sharedFallback
              ? __webpack_require__.federation.bundlerRuntime?.getSharedFallbackGetter(
                  {
                    shareKey: data.shareKey,
                    factory: data.fallback,
                    webpackRequire: __webpack_require__,
                    libraryType: __webpack_require__.federation.libraryType,
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
            treeShaking: __webpack_require__.federation.sharedFallback
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

    early(__webpack_require__.federation, 'initOptions', () => ({}));
    early(
      __webpack_require__.federation.initOptions,
      'name',
      () => __module_federation_container_name__,
    );
    early(
      __webpack_require__.federation.initOptions,
      'shareStrategy',
      () => __module_federation_share_strategy__,
    );
    early(__webpack_require__.federation.initOptions, 'shared', () => {
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
    merge(__webpack_require__.federation.initOptions, 'remotes', () =>
      Object.values(__module_federation_remote_infos__)
        .flat()
        .filter((remote) => remote.externalType === 'script')
        .flatMap(expandRemoteScopes),
    );
    merge(
      __webpack_require__.federation.initOptions,
      'plugins',
      () => __module_federation_runtime_plugins__,
    );

    early(__webpack_require__.federation, 'bundlerRuntimeOptions', () => ({}));
    early(
      __webpack_require__.federation.bundlerRuntimeOptions,
      'remotes',
      () => ({}),
    );
    early(
      __webpack_require__.federation.bundlerRuntimeOptions.remotes,
      'chunkMapping',
      () => remotesLoadingChunkMapping,
    );
    early(
      __webpack_require__.federation.bundlerRuntimeOptions.remotes,
      'remoteInfos',
      () => bundlerRuntimeRemoteInfos,
    );
    early(
      __webpack_require__.federation.bundlerRuntimeOptions.remotes,
      'idToExternalAndNameMapping',
      () => {
        const remotesLoadingIdToExternalAndNameMappingMapping = {};
        for (let [moduleId, data] of Object.entries(
          remotesLoadingModuleIdToRemoteDataMapping,
        )) {
          remotesLoadingIdToExternalAndNameMappingMapping[moduleId] = [
            toRuntimeScope(data.shareScope),
            data.name,
            data.externalModuleId,
            data.remoteName,
          ];
        }
        return remotesLoadingIdToExternalAndNameMappingMapping;
      },
    );
    early(
      __webpack_require__.federation.bundlerRuntimeOptions.remotes,
      'webpackRequire',
      () => __webpack_require__,
    );
    merge(
      __webpack_require__.federation.bundlerRuntimeOptions.remotes,
      'idToRemoteMap',
      () => {
        const idToRemoteMap = {};
        for (let [id, remoteData] of Object.entries(
          remotesLoadingModuleIdToRemoteDataMapping,
        )) {
          const info = bundlerRuntimeRemoteInfos[remoteData.remoteName];
          if (info) idToRemoteMap[id] = info;
        }
        return idToRemoteMap;
      },
    );

    override(
      __webpack_require__,
      'S',
      __webpack_require__.federation.bundlerRuntime.S,
    );
    if (__webpack_require__.federation.attachShareScopeMap) {
      __webpack_require__.federation.attachShareScopeMap(__webpack_require__);
    }

    override(__webpack_require__.f, 'remotes', (chunkId, promises) =>
      __webpack_require__.federation.bundlerRuntime.remotes({
        chunkId,
        promises,
        chunkMapping: remotesLoadingChunkMapping,
        idToExternalAndNameMapping:
          __webpack_require__.federation.bundlerRuntimeOptions.remotes
            .idToExternalAndNameMapping,
        idToRemoteMap:
          __webpack_require__.federation.bundlerRuntimeOptions.remotes
            .idToRemoteMap,
        webpackRequire: __webpack_require__,
      }),
    );
    override(__webpack_require__.f, 'consumes', (chunkId, promises) => {
      const consumeModuleIds = consumesLoadingChunkMapping[chunkId];
      const consumeWithBundlerRuntime = (targetPromises) =>
        __webpack_require__.federation.bundlerRuntime.consumes({
          chunkId,
          promises: targetPromises,
          chunkMapping: consumesLoadingChunkMapping,
          moduleToHandlerMapping:
            __webpack_require__.federation
              .consumesLoadingModuleToHandlerMapping,
          installedModules: consumesLoadinginstalledModules,
          webpackRequire: __webpack_require__,
        });
      if (Array.isArray(consumeModuleIds) && consumeModuleIds.length > 0) {
        const initPromises = [];
        const initScopeSet = new Set();
        for (const moduleId of consumeModuleIds) {
          const consumeData =
            consumesLoadingModuleToConsumeDataMapping[moduleId];
          if (!consumeData) continue;
          if (consumeData.import == null || consumeData.layer == null) {
            for (const scope of toScopeArray(consumeData.shareScope)) {
              initScopeSet.add(scope);
            }
          }
        }
        for (const scope of initScopeSet) {
          const init = __webpack_require__.I(scope);
          if (init && init.then) {
            initPromises.push(init);
          }
        }
        if (initPromises.length > 0) {
          promises.push(
            Promise.all(initPromises).then(() => {
              const consumePromises = [];
              consumeWithBundlerRuntime(consumePromises);
              return Promise.all(consumePromises);
            }),
          );
          return;
        }
      }
      return consumeWithBundlerRuntime(promises);
    });
    override(__webpack_require__, 'I', (name, initScope) =>
      __webpack_require__.federation.bundlerRuntime.I({
        shareScopeName: name,
        initScope,
        initPromises: initializeSharingInitPromises,
        initTokens: initializeSharingInitTokens,
        webpackRequire: __webpack_require__,
      }),
    );
    override(
      __webpack_require__,
      'initContainer',
      (shareScope, initScope, remoteEntryInitOptions) =>
        __webpack_require__.federation.bundlerRuntime.initContainerEntry({
          shareScope,
          initScope,
          remoteEntryInitOptions,
          shareScopeKey: containerShareScope,
          webpackRequire: __webpack_require__,
        }),
    );
    override(__webpack_require__, 'getContainer', (module, getScope) => {
      var moduleMap = __webpack_require__.initializeExposesData.moduleMap;
      __webpack_require__.R = getScope;
      getScope = Object.prototype.hasOwnProperty.call(moduleMap, module)
        ? moduleMap[module]()
        : Promise.resolve().then(() => {
            throw new Error(
              'Module "' + module + '" does not exist in container.',
            );
          });
      __webpack_require__.R = undefined;
      return getScope;
    });

    __webpack_require__.federation.instance =
      __webpack_require__.federation.bundlerRuntime.init({
        webpackRequire: __webpack_require__,
      });

    if (__webpack_require__.consumesLoadingData?.initialConsumes) {
      __webpack_require__.federation.bundlerRuntime.installInitialConsumes({
        webpackRequire: __webpack_require__,
        installedModules: consumesLoadinginstalledModules,
        initialConsumes:
          __webpack_require__.consumesLoadingData.initialConsumes,
        moduleToHandlerMapping:
          __webpack_require__.federation.consumesLoadingModuleToHandlerMapping,
      });
    }
  }
}
