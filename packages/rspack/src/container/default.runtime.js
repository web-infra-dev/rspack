// @ts-nocheck
var __module_federation_runtime__,
	__module_federation_runtime_plugins__,
	__module_federation_remote_infos__;
module.exports = function () {
	if (
		__webpack_require__.initializeSharingData ||
		__webpack_require__.initializeExposesData
	) {
		const scopeToInitDataMapping =
			__webpack_require__.initializeSharingData?.scopeToSharingDataMapping ??
			{};
		const shared = {};
		for (let [scope, stages] of Object.entries(scopeToInitDataMapping)) {
			for (let stage of stages) {
				if (Array.isArray(stage)) {
					const [name, version, factory, eager] = stage;
					if (shared[name]) {
						shared[name].scope.push(scope);
					} else {
						shared[name] = { version, get: factory, scope: [scope] };
					}
				}
			}
		}

		const remotesLoadingModuleIdToRemoteDataMapping =
			__webpack_require__.remotesLoadingData?.moduleIdToRemoteDataMapping ?? {};
		const idToRemoteMap = {};
		for (let [id, remoteData] of Object.entries(
			remotesLoadingModuleIdToRemoteDataMapping
		)) {
			const info = __module_federation_remote_infos__[remoteData[3]];
			if (info) idToRemoteMap[id] = info;
		}

		const moduleToConsumeDataMapping =
			__webpack_require__.consumesLoadingData?.moduleIdToConsumeDataMapping ??
			{};
		const consumesLoadingModuleToHandlerMapping = {};
		for (let [moduleId, data] of Object.entries(moduleToConsumeDataMapping)) {
			consumesLoadingModuleToHandlerMapping[moduleId] = {
				getter: data.fallback,
				shareInfo: {
					shareConfig: {
						fixedDependencies: false,
						requiredVersion: data.requiredVersion,
						strictVersion: data.strictVersion,
						singleton: data.singleton,
						eager: data.eager
					},
					scope: [data.shareScope]
				},
				shareKey: data.shareKey
			};
		}

		const consumesLoadinginstalledModules = {};
		const initializeSharingInitPromises = [];
		const initializeSharingInitTokens = [];
		const remotesLoadingChunkMapping =
			__webpack_require__.remotesLoadingData?.chunkMapping ?? {};
		const consumesLoadingChunkMapping =
			__webpack_require__.consumesLoadingData?.chunkMapping ?? {};
		const containerShareScope =
			__webpack_require__.initializeExposesData?.containerShareScope;

		__module_federation_runtime__.initOptions = {
			name: __webpack_require__.initializeSharingData?.uniqueName,
			remotes: Object.values(__module_federation_remote_infos__).filter(
				remote => remote.externalType === "script"
			),
			shared: shared,
			plugins: __module_federation_runtime_plugins__
		};
		__module_federation_runtime__.bundlerRuntimeOptions = {
			remotes: {
				idToRemoteMap,
				chunkMapping: remotesLoadingChunkMapping,
				idToExternalAndNameMapping: remotesLoadingModuleIdToRemoteDataMapping,
				webpackRequire: __webpack_require__
			}
		};

		if (__webpack_require__.f?.remotes) {
			__webpack_require__.f.remotes = (chunkId, promises) =>
				__module_federation_runtime__.bundlerRuntime.remotes({
					chunkId,
					promises,
					chunkMapping: remotesLoadingChunkMapping,
					idToExternalAndNameMapping: remotesLoadingModuleIdToRemoteDataMapping,
					idToRemoteMap,
					webpackRequire: __webpack_require__
				});
		}
		if (__webpack_require__.f?.consumes) {
			__webpack_require__.f.consumes = (chunkId, promises) =>
				__module_federation_runtime__.bundlerRuntime.consumes({
					chunkId,
					promises,
					chunkMapping: consumesLoadingChunkMapping,
					moduleToHandlerMapping: consumesLoadingModuleToHandlerMapping,
					installedModules: consumesLoadinginstalledModules,
					webpackRequire: __webpack_require__
				});
		}
		if (__webpack_require__.I) {
			__webpack_require__.I = (name, initScope) =>
				__module_federation_runtime__.bundlerRuntime.I({
					shareScopeName: name,
					initScope,
					initPromises: initializeSharingInitPromises,
					initTokens: initializeSharingInitTokens,
					webpackRequire: __webpack_require__
				});
		}
		if (__webpack_require__.S) {
			__webpack_require__.S = __module_federation_runtime__.bundlerRuntime.S;
		}
		if (__webpack_require__.initContainer) {
			__webpack_require__.initContainer = (shareScope, initScope) =>
				__module_federation_runtime__.bundlerRuntime.initContainerEntry({
					shareScope,
					initScope,
					shareScopeKey: containerShareScope,
					webpackRequire: __webpack_require__
				});
		}
		if (__webpack_require__.getContainer) {
			__webpack_require__.getContainer = (module, getScope) => {
				var moduleMap = __webpack_require__.initializeExposesData.moduleMap;
				__webpack_require__.R = getScope;
				getScope = Object.prototype.hasOwnProperty.call(moduleMap, module)
					? moduleMap[module]()
					: Promise.resolve().then(() => {
							throw new Error(
								'Module "' + module + '" does not exist in container.'
							);
					  });
				__webpack_require__.R = undefined;
				return getScope;
			};
		}

		__webpack_require__.federation = __module_federation_runtime__;

		__module_federation_runtime__.instance =
			__module_federation_runtime__.runtime.init(
				__module_federation_runtime__.initOptions
			);

		if (__webpack_require__.consumesLoadingData?.initialConsumes) {
			__module_federation_runtime__.bundlerRuntime.installInitialConsumes({
				webpackRequire: __webpack_require__,
				installedModules: consumesLoadinginstalledModules,
				initialConsumes:
					__webpack_require__.consumesLoadingData.initialConsumes,
				moduleToHandlerMapping: consumesLoadingModuleToHandlerMapping
			});
		}
	}
};
