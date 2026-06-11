// @ts-nocheck
var __module_federation_bundler_runtime__,
  __module_federation_runtime_plugins__,
  __module_federation_remote_infos__,
  __module_federation_container_name__,
  __module_federation_share_strategy__,
  __module_federation_share_fallbacks__,
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
    const containerShareScope =
      runtimeRequire.initializeExposesData?.shareScope;

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
    early(
      runtimeRequire.federation,
      'consumesLoadingModuleToHandlerMapping',
      () => {
        const consumesLoadingModuleToHandlerMapping = {};
        for (let [moduleId, data] of Object.entries(
          consumesLoadingModuleToConsumeDataMapping,
        )) {
          consumesLoadingModuleToHandlerMapping[moduleId] = {
            getter: sharedFallback
              ? runtimeRequire.federation.bundlerRuntime?.getSharedFallbackGetter(
                  {
                    shareKey: data.shareKey,
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
              },
              scope: [data.shareScope],
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
          const info =
            __module_federation_remote_infos__[remoteData.remoteName];
          if (info) idToRemoteMap[id] = info;
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
    override(runtimeRequire.f, 'consumes', (chunkId, promises) =>
      runtimeRequire.federation.bundlerRuntime.consumes({
        chunkId,
        promises,
        chunkMapping: consumesLoadingChunkMapping,
        moduleToHandlerMapping:
          runtimeRequire.federation.consumesLoadingModuleToHandlerMapping,
        installedModules: consumesLoadinginstalledModules,
        webpackRequire: runtimeRequire,
      }),
    );
    override(runtimeRequire, 'I', (name, initScope) =>
      runtimeRequire.federation.bundlerRuntime.I({
        shareScopeName: name,
        initScope,
        initPromises: initializeSharingInitPromises,
        initTokens: initializeSharingInitTokens,
        webpackRequire: runtimeRequire,
      }),
    );
    override(
      runtimeRequire,
      'initContainer',
      (shareScope, initScope, remoteEntryInitOptions) =>
        runtimeRequire.federation.bundlerRuntime.initContainerEntry({
          shareScope,
          initScope,
          remoteEntryInitOptions,
          shareScopeKey: containerShareScope,
          webpackRequire: runtimeRequire,
        }),
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

    if (runtimeRequire.consumesLoadingData?.initialConsumes) {
      runtimeRequire.federation.bundlerRuntime.installInitialConsumes({
        webpackRequire: runtimeRequire,
        installedModules: consumesLoadinginstalledModules,
        initialConsumes: runtimeRequire.consumesLoadingData.initialConsumes,
        moduleToHandlerMapping:
          runtimeRequire.federation.consumesLoadingModuleToHandlerMapping,
      });
    }
  }
}
