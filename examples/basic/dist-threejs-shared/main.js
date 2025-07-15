(() => {
	// webpackBootstrap
	"use strict";
	var __webpack_modules__ = {
		"../../node_modules/.pnpm/@module-federation+error-codes@0.16.0/node_modules/@module-federation/error-codes/dist/index.cjs.js":
			/*!************************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+error-codes@0.16.0/node_modules/@module-federation/error-codes/dist/index.cjs.js ***!
  \************************************************************************************************************************************/
			function (__unused_webpack_module, exports) {
				const RUNTIME_001 = "RUNTIME-001";
				const RUNTIME_002 = "RUNTIME-002";
				const RUNTIME_003 = "RUNTIME-003";
				const RUNTIME_004 = "RUNTIME-004";
				const RUNTIME_005 = "RUNTIME-005";
				const RUNTIME_006 = "RUNTIME-006";
				const RUNTIME_007 = "RUNTIME-007";
				const RUNTIME_008 = "RUNTIME-008";
				const TYPE_001 = "TYPE-001";
				const BUILD_001 = "BUILD-001";
				const BUILD_002 = "BUILD-002";

				const getDocsUrl = errorCode => {
					const type = errorCode.split("-")[0].toLowerCase();
					return `View the docs to see how to solve: https://module-federation.io/guide/troubleshooting/${type}/${errorCode}`;
				};
				const getShortErrorMsg = (
					errorCode,
					errorDescMap,
					args,
					originalErrorMsg
				) => {
					const msg = [`${[errorDescMap[errorCode]]} #${errorCode}`];
					args && msg.push(`args: ${JSON.stringify(args)}`);
					msg.push(getDocsUrl(errorCode));
					originalErrorMsg &&
						msg.push(`Original Error Message:\n ${originalErrorMsg}`);
					return msg.join("\n");
				};

				function _extends() {
					_extends =
						Object.assign ||
						function assign(target) {
							for (var i = 1; i < arguments.length; i++) {
								var source = arguments[i];
								for (var key in source)
									if (Object.prototype.hasOwnProperty.call(source, key))
										target[key] = source[key];
							}
							return target;
						};
					return _extends.apply(this, arguments);
				}

				const runtimeDescMap = {
					[RUNTIME_001]: "Failed to get remoteEntry exports.",
					[RUNTIME_002]: 'The remote entry interface does not contain "init"',
					[RUNTIME_003]: "Failed to get manifest.",
					[RUNTIME_004]: "Failed to locate remote.",
					[RUNTIME_005]:
						"Invalid loadShareSync function call from bundler runtime",
					[RUNTIME_006]: "Invalid loadShareSync function call from runtime",
					[RUNTIME_007]: "Failed to get remote snapshot.",
					[RUNTIME_008]: "Failed to load script resources."
				};
				const typeDescMap = {
					[TYPE_001]:
						"Failed to generate type declaration. Execute the below cmd to reproduce and fix the error."
				};
				const buildDescMap = {
					[BUILD_001]: "Failed to find expose module.",
					[BUILD_002]: "PublicPath is required in prod mode."
				};
				const errorDescMap = _extends(
					{},
					runtimeDescMap,
					typeDescMap,
					buildDescMap
				);

				exports.BUILD_001 = BUILD_001;
				exports.BUILD_002 = BUILD_002;
				exports.RUNTIME_001 = RUNTIME_001;
				exports.RUNTIME_002 = RUNTIME_002;
				exports.RUNTIME_003 = RUNTIME_003;
				exports.RUNTIME_004 = RUNTIME_004;
				exports.RUNTIME_005 = RUNTIME_005;
				exports.RUNTIME_006 = RUNTIME_006;
				exports.RUNTIME_007 = RUNTIME_007;
				exports.RUNTIME_008 = RUNTIME_008;
				exports.TYPE_001 = TYPE_001;
				exports.buildDescMap = buildDescMap;
				exports.errorDescMap = errorDescMap;
				exports.getShortErrorMsg = getShortErrorMsg;
				exports.runtimeDescMap = runtimeDescMap;
				exports.typeDescMap = typeDescMap;
			},
		'@module-federation/runtime/rspack.js!=!data:text/javascript,import __module_federation_bundler_runtime__ from "/Users/bytedance/RustroverProjects/rspack/node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs";const __module_federation_runtime_plugins__ = [];const __module_federation_remote_infos__ = {};const __module_federation_container_name__ = "threejs_app";const __module_federation_share_strategy__ = "version-first";if((__webpack_require__.initializeSharingData||__webpack_require__.initializeExposesData)&&__webpack_require__.federation){var __webpack_require___remotesLoadingData,__webpack_require___remotesLoadingData1,__webpack_require___initializeSharingData,__webpack_require___consumesLoadingData,__webpack_require___consumesLoadingData1,__webpack_require___initializeExposesData,__webpack_require___consumesLoadingData2;const override=(obj,key,value)=>{if(!obj)return;if(obj[key])obj[key]=value};const merge=(obj,key,fn)=>{const value=fn();if(Array.isArray(value)){var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=[];obj[key].push(...value)}else if(typeof value==="object"&&value!==null){var _obj1,_key1;var _1;(_1=(_obj1=obj)[_key1=key])!==null&&_1!==void 0?_1:_obj1[_key1]={};Object.assign(obj[key],value)}};const early=(obj,key,initial)=>{var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=initial()};var __webpack_require___remotesLoadingData_chunkMapping;const remotesLoadingChunkMapping=(__webpack_require___remotesLoadingData_chunkMapping=(__webpack_require___remotesLoadingData=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData===void 0?void 0:__webpack_require___remotesLoadingData.chunkMapping)!==null&&__webpack_require___remotesLoadingData_chunkMapping!==void 0?__webpack_require___remotesLoadingData_chunkMapping:{};var __webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping;const remotesLoadingModuleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData1=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData1===void 0?void 0:__webpack_require___remotesLoadingData1.moduleIdToRemoteDataMapping)!==null&&__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping!==void 0?__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping:{};var __webpack_require___initializeSharingData_scopeToSharingDataMapping;const initializeSharingScopeToInitDataMapping=(__webpack_require___initializeSharingData_scopeToSharingDataMapping=(__webpack_require___initializeSharingData=__webpack_require__.initializeSharingData)===null||__webpack_require___initializeSharingData===void 0?void 0:__webpack_require___initializeSharingData.scopeToSharingDataMapping)!==null&&__webpack_require___initializeSharingData_scopeToSharingDataMapping!==void 0?__webpack_require___initializeSharingData_scopeToSharingDataMapping:{};var __webpack_require___consumesLoadingData_chunkMapping;const consumesLoadingChunkMapping=(__webpack_require___consumesLoadingData_chunkMapping=(__webpack_require___consumesLoadingData=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData===void 0?void 0:__webpack_require___consumesLoadingData.chunkMapping)!==null&&__webpack_require___consumesLoadingData_chunkMapping!==void 0?__webpack_require___consumesLoadingData_chunkMapping:{};var __webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping;const consumesLoadingModuleToConsumeDataMapping=(__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping=(__webpack_require___consumesLoadingData1=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData1===void 0?void 0:__webpack_require___consumesLoadingData1.moduleIdToConsumeDataMapping)!==null&&__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping!==void 0?__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping:{};const consumesLoadinginstalledModules={};const initializeSharingInitPromises=[];const initializeSharingInitTokens={};const containerShareScope=(__webpack_require___initializeExposesData=__webpack_require__.initializeExposesData)===null||__webpack_require___initializeExposesData===void 0?void 0:__webpack_require___initializeExposesData.shareScope;for(const key in __module_federation_bundler_runtime__){__webpack_require__.federation[key]=__module_federation_bundler_runtime__[key]}early(__webpack_require__.federation,"consumesLoadingModuleToHandlerMapping",()=>{const consumesLoadingModuleToHandlerMapping={};for(let[moduleId,data]of Object.entries(consumesLoadingModuleToConsumeDataMapping)){consumesLoadingModuleToHandlerMapping[moduleId]={getter:data.fallback,shareInfo:{shareConfig:{fixedDependencies:false,requiredVersion:data.requiredVersion,strictVersion:data.strictVersion,singleton:data.singleton,eager:data.eager},scope:[data.shareScope]},shareKey:data.shareKey}}return consumesLoadingModuleToHandlerMapping});early(__webpack_require__.federation,"initOptions",()=>({}));early(__webpack_require__.federation.initOptions,"name",()=>__module_federation_container_name__);early(__webpack_require__.federation.initOptions,"shareStrategy",()=>__module_federation_share_strategy__);early(__webpack_require__.federation.initOptions,"shared",()=>{const shared={};for(let[scope,stages]of Object.entries(initializeSharingScopeToInitDataMapping)){for(let stage of stages){if(typeof stage==="object"&&stage!==null){const{name,version,factory,eager,singleton,requiredVersion,strictVersion}=stage;const shareConfig={};const isValidValue=function(val){return typeof val!=="undefined"};if(isValidValue(singleton)){shareConfig.singleton=singleton}if(isValidValue(requiredVersion)){shareConfig.requiredVersion=requiredVersion}if(isValidValue(eager)){shareConfig.eager=eager}if(isValidValue(strictVersion)){shareConfig.strictVersion=strictVersion}const options={version,scope:[scope],shareConfig,get:factory};if(shared[name]){shared[name].push(options)}else{shared[name]=[options]}}}}return shared});merge(__webpack_require__.federation.initOptions,"remotes",()=>Object.values(__module_federation_remote_infos__).flat().filter(remote=>remote.externalType==="script"));merge(__webpack_require__.federation.initOptions,"plugins",()=>__module_federation_runtime_plugins__);early(__webpack_require__.federation,"bundlerRuntimeOptions",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions,"remotes",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"chunkMapping",()=>remotesLoadingChunkMapping);early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToExternalAndNameMapping",()=>{const remotesLoadingIdToExternalAndNameMappingMapping={};for(let[moduleId,data]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){remotesLoadingIdToExternalAndNameMappingMapping[moduleId]=[data.shareScope,data.name,data.externalModuleId,data.remoteName]}return remotesLoadingIdToExternalAndNameMappingMapping});early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"webpackRequire",()=>__webpack_require__);merge(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToRemoteMap",()=>{const idToRemoteMap={};for(let[id,remoteData]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){const info=__module_federation_remote_infos__[remoteData.remoteName];if(info)idToRemoteMap[id]=info}return idToRemoteMap});override(__webpack_require__,"S",__webpack_require__.federation.bundlerRuntime.S);if(__webpack_require__.federation.attachShareScopeMap){__webpack_require__.federation.attachShareScopeMap(__webpack_require__)}override(__webpack_require__.f,"remotes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.remotes({chunkId,promises,chunkMapping:remotesLoadingChunkMapping,idToExternalAndNameMapping:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToExternalAndNameMapping,idToRemoteMap:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToRemoteMap,webpackRequire:__webpack_require__}));override(__webpack_require__.f,"consumes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.consumes({chunkId,promises,chunkMapping:consumesLoadingChunkMapping,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping,installedModules:consumesLoadinginstalledModules,webpackRequire:__webpack_require__}));override(__webpack_require__,"I",(name,initScope)=>__webpack_require__.federation.bundlerRuntime.I({shareScopeName:name,initScope,initPromises:initializeSharingInitPromises,initTokens:initializeSharingInitTokens,webpackRequire:__webpack_require__}));override(__webpack_require__,"initContainer",(shareScope,initScope,remoteEntryInitOptions)=>__webpack_require__.federation.bundlerRuntime.initContainerEntry({shareScope,initScope,remoteEntryInitOptions,shareScopeKey:containerShareScope,webpackRequire:__webpack_require__}));override(__webpack_require__,"getContainer",(module1,getScope)=>{var moduleMap=__webpack_require__.initializeExposesData.moduleMap;__webpack_require__.R=getScope;getScope=Object.prototype.hasOwnProperty.call(moduleMap,module1)?moduleMap[module1]():Promise.resolve().then(()=>{throw new Error(\'Module "\'+module1+\'" does not exist in container.\')});__webpack_require__.R=undefined;return getScope});__webpack_require__.federation.instance=__webpack_require__.federation.runtime.init(__webpack_require__.federation.initOptions);if((__webpack_require___consumesLoadingData2=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData2===void 0?void 0:__webpack_require___consumesLoadingData2.initialConsumes){__webpack_require__.federation.bundlerRuntime.installInitialConsumes({webpackRequire:__webpack_require__,installedModules:consumesLoadinginstalledModules,initialConsumes:__webpack_require__.consumesLoadingData.initialConsumes,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping})}}':
			/*!***********************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************!*\
  !*** @module-federation/runtime/rspack.js!=!data:text/javascript,import __module_federation_bundler_runtime__ from "/Users/bytedance/RustroverProjects/rspack/node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs";const __module_federation_runtime_plugins__ = [];const __module_federation_remote_infos__ = {};const __module_federation_container_name__ = "threejs_app";const __module_federation_share_strategy__ = "version-first";if((__webpack_require__.initializeSharingData||__webpack_require__.initializeExposesData)&&__webpack_require__.federation){var __webpack_require___remotesLoadingData,__webpack_require___remotesLoadingData1,__webpack_require___initializeSharingData,__webpack_require___consumesLoadingData,__webpack_require___consumesLoadingData1,__webpack_require___initializeExposesData,__webpack_require___consumesLoadingData2;const override=(obj,key,value)=>{if(!obj)return;if(obj[key])obj[key]=value};const merge=(obj,key,fn)=>{const value=fn();if(Array.isArray(value)){var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=[];obj[key].push(...value)}else if(typeof value==="object"&&value!==null){var _obj1,_key1;var _1;(_1=(_obj1=obj)[_key1=key])!==null&&_1!==void 0?_1:_obj1[_key1]={};Object.assign(obj[key],value)}};const early=(obj,key,initial)=>{var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=initial()};var __webpack_require___remotesLoadingData_chunkMapping;const remotesLoadingChunkMapping=(__webpack_require___remotesLoadingData_chunkMapping=(__webpack_require___remotesLoadingData=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData===void 0?void 0:__webpack_require___remotesLoadingData.chunkMapping)!==null&&__webpack_require___remotesLoadingData_chunkMapping!==void 0?__webpack_require___remotesLoadingData_chunkMapping:{};var __webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping;const remotesLoadingModuleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData1=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData1===void 0?void 0:__webpack_require___remotesLoadingData1.moduleIdToRemoteDataMapping)!==null&&__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping!==void 0?__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping:{};var __webpack_require___initializeSharingData_scopeToSharingDataMapping;const initializeSharingScopeToInitDataMapping=(__webpack_require___initializeSharingData_scopeToSharingDataMapping=(__webpack_require___initializeSharingData=__webpack_require__.initializeSharingData)===null||__webpack_require___initializeSharingData===void 0?void 0:__webpack_require___initializeSharingData.scopeToSharingDataMapping)!==null&&__webpack_require___initializeSharingData_scopeToSharingDataMapping!==void 0?__webpack_require___initializeSharingData_scopeToSharingDataMapping:{};var __webpack_require___consumesLoadingData_chunkMapping;const consumesLoadingChunkMapping=(__webpack_require___consumesLoadingData_chunkMapping=(__webpack_require___consumesLoadingData=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData===void 0?void 0:__webpack_require___consumesLoadingData.chunkMapping)!==null&&__webpack_require___consumesLoadingData_chunkMapping!==void 0?__webpack_require___consumesLoadingData_chunkMapping:{};var __webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping;const consumesLoadingModuleToConsumeDataMapping=(__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping=(__webpack_require___consumesLoadingData1=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData1===void 0?void 0:__webpack_require___consumesLoadingData1.moduleIdToConsumeDataMapping)!==null&&__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping!==void 0?__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping:{};const consumesLoadinginstalledModules={};const initializeSharingInitPromises=[];const initializeSharingInitTokens={};const containerShareScope=(__webpack_require___initializeExposesData=__webpack_require__.initializeExposesData)===null||__webpack_require___initializeExposesData===void 0?void 0:__webpack_require___initializeExposesData.shareScope;for(const key in __module_federation_bundler_runtime__){__webpack_require__.federation[key]=__module_federation_bundler_runtime__[key]}early(__webpack_require__.federation,"consumesLoadingModuleToHandlerMapping",()=>{const consumesLoadingModuleToHandlerMapping={};for(let[moduleId,data]of Object.entries(consumesLoadingModuleToConsumeDataMapping)){consumesLoadingModuleToHandlerMapping[moduleId]={getter:data.fallback,shareInfo:{shareConfig:{fixedDependencies:false,requiredVersion:data.requiredVersion,strictVersion:data.strictVersion,singleton:data.singleton,eager:data.eager},scope:[data.shareScope]},shareKey:data.shareKey}}return consumesLoadingModuleToHandlerMapping});early(__webpack_require__.federation,"initOptions",()=>({}));early(__webpack_require__.federation.initOptions,"name",()=>__module_federation_container_name__);early(__webpack_require__.federation.initOptions,"shareStrategy",()=>__module_federation_share_strategy__);early(__webpack_require__.federation.initOptions,"shared",()=>{const shared={};for(let[scope,stages]of Object.entries(initializeSharingScopeToInitDataMapping)){for(let stage of stages){if(typeof stage==="object"&&stage!==null){const{name,version,factory,eager,singleton,requiredVersion,strictVersion}=stage;const shareConfig={};const isValidValue=function(val){return typeof val!=="undefined"};if(isValidValue(singleton)){shareConfig.singleton=singleton}if(isValidValue(requiredVersion)){shareConfig.requiredVersion=requiredVersion}if(isValidValue(eager)){shareConfig.eager=eager}if(isValidValue(strictVersion)){shareConfig.strictVersion=strictVersion}const options={version,scope:[scope],shareConfig,get:factory};if(shared[name]){shared[name].push(options)}else{shared[name]=[options]}}}}return shared});merge(__webpack_require__.federation.initOptions,"remotes",()=>Object.values(__module_federation_remote_infos__).flat().filter(remote=>remote.externalType==="script"));merge(__webpack_require__.federation.initOptions,"plugins",()=>__module_federation_runtime_plugins__);early(__webpack_require__.federation,"bundlerRuntimeOptions",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions,"remotes",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"chunkMapping",()=>remotesLoadingChunkMapping);early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToExternalAndNameMapping",()=>{const remotesLoadingIdToExternalAndNameMappingMapping={};for(let[moduleId,data]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){remotesLoadingIdToExternalAndNameMappingMapping[moduleId]=[data.shareScope,data.name,data.externalModuleId,data.remoteName]}return remotesLoadingIdToExternalAndNameMappingMapping});early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"webpackRequire",()=>__webpack_require__);merge(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToRemoteMap",()=>{const idToRemoteMap={};for(let[id,remoteData]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){const info=__module_federation_remote_infos__[remoteData.remoteName];if(info)idToRemoteMap[id]=info}return idToRemoteMap});override(__webpack_require__,"S",__webpack_require__.federation.bundlerRuntime.S);if(__webpack_require__.federation.attachShareScopeMap){__webpack_require__.federation.attachShareScopeMap(__webpack_require__)}override(__webpack_require__.f,"remotes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.remotes({chunkId,promises,chunkMapping:remotesLoadingChunkMapping,idToExternalAndNameMapping:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToExternalAndNameMapping,idToRemoteMap:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToRemoteMap,webpackRequire:__webpack_require__}));override(__webpack_require__.f,"consumes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.consumes({chunkId,promises,chunkMapping:consumesLoadingChunkMapping,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping,installedModules:consumesLoadinginstalledModules,webpackRequire:__webpack_require__}));override(__webpack_require__,"I",(name,initScope)=>__webpack_require__.federation.bundlerRuntime.I({shareScopeName:name,initScope,initPromises:initializeSharingInitPromises,initTokens:initializeSharingInitTokens,webpackRequire:__webpack_require__}));override(__webpack_require__,"initContainer",(shareScope,initScope,remoteEntryInitOptions)=>__webpack_require__.federation.bundlerRuntime.initContainerEntry({shareScope,initScope,remoteEntryInitOptions,shareScopeKey:containerShareScope,webpackRequire:__webpack_require__}));override(__webpack_require__,"getContainer",(module1,getScope)=>{var moduleMap=__webpack_require__.initializeExposesData.moduleMap;__webpack_require__.R=getScope;getScope=Object.prototype.hasOwnProperty.call(moduleMap,module1)?moduleMap[module1]():Promise.resolve().then(()=>{throw new Error('Module "'+module1+'" does not exist in container.')});__webpack_require__.R=undefined;return getScope});__webpack_require__.federation.instance=__webpack_require__.federation.runtime.init(__webpack_require__.federation.initOptions);if((__webpack_require___consumesLoadingData2=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData2===void 0?void 0:__webpack_require___consumesLoadingData2.initialConsumes){__webpack_require__.federation.bundlerRuntime.installInitialConsumes({webpackRequire:__webpack_require__,installedModules:consumesLoadinginstalledModules,initialConsumes:__webpack_require__.consumesLoadingData.initialConsumes,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping})}} ***!
  \***********************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************************/
			function (
				__unused_webpack_module,
				__unused_webpack___webpack_exports__,
				__webpack_require__
			) {
				/* ESM import */ var _Users_bytedance_RustroverProjects_rspack_node_modules_pnpm_module_federation_webpack_bundler_runtime_0_16_0_node_modules_module_federation_webpack_bundler_runtime_dist_index_cjs_cjs__WEBPACK_IMPORTED_MODULE_0__ =
					__webpack_require__(
						/*! ../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs */ "../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs"
					);
				/* ESM import */ var _Users_bytedance_RustroverProjects_rspack_node_modules_pnpm_module_federation_webpack_bundler_runtime_0_16_0_node_modules_module_federation_webpack_bundler_runtime_dist_index_cjs_cjs__WEBPACK_IMPORTED_MODULE_0___default =
					/*#__PURE__*/ __webpack_require__.n(
						_Users_bytedance_RustroverProjects_rspack_node_modules_pnpm_module_federation_webpack_bundler_runtime_0_16_0_node_modules_module_federation_webpack_bundler_runtime_dist_index_cjs_cjs__WEBPACK_IMPORTED_MODULE_0__
					);
				const __module_federation_runtime_plugins__ = [];
				const __module_federation_remote_infos__ = {};
				const __module_federation_container_name__ = "threejs_app";
				const __module_federation_share_strategy__ = "version-first";
				if (
					(__webpack_require__.initializeSharingData ||
						__webpack_require__.initializeExposesData) &&
					__webpack_require__.federation
				) {
					var __webpack_require___remotesLoadingData,
						__webpack_require___remotesLoadingData1,
						__webpack_require___initializeSharingData,
						__webpack_require___consumesLoadingData,
						__webpack_require___consumesLoadingData1,
						__webpack_require___initializeExposesData,
						__webpack_require___consumesLoadingData2;
					const override = (obj, key, value) => {
						if (!obj) return;
						if (obj[key]) obj[key] = value;
					};
					const merge = (obj, key, fn) => {
						const value = fn();
						if (Array.isArray(value)) {
							var _obj, _key;
							var _;
							(_ = (_obj = obj)[(_key = key)]) !== null && _ !== void 0
								? _
								: (_obj[_key] = []);
							obj[key].push(...value);
						} else if (typeof value === "object" && value !== null) {
							var _obj1, _key1;
							var _1;
							(_1 = (_obj1 = obj)[(_key1 = key)]) !== null && _1 !== void 0
								? _1
								: (_obj1[_key1] = {});
							Object.assign(obj[key], value);
						}
					};
					const early = (obj, key, initial) => {
						var _obj, _key;
						var _;
						(_ = (_obj = obj)[(_key = key)]) !== null && _ !== void 0
							? _
							: (_obj[_key] = initial());
					};
					var __webpack_require___remotesLoadingData_chunkMapping;
					const remotesLoadingChunkMapping =
						(__webpack_require___remotesLoadingData_chunkMapping =
							(__webpack_require___remotesLoadingData =
								__webpack_require__.remotesLoadingData) === null ||
							__webpack_require___remotesLoadingData === void 0
								? void 0
								: __webpack_require___remotesLoadingData.chunkMapping) !==
							null &&
						__webpack_require___remotesLoadingData_chunkMapping !== void 0
							? __webpack_require___remotesLoadingData_chunkMapping
							: {};
					var __webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping;
					const remotesLoadingModuleIdToRemoteDataMapping =
						(__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping =
							(__webpack_require___remotesLoadingData1 =
								__webpack_require__.remotesLoadingData) === null ||
							__webpack_require___remotesLoadingData1 === void 0
								? void 0
								: __webpack_require___remotesLoadingData1.moduleIdToRemoteDataMapping) !==
							null &&
						__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping !==
							void 0
							? __webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping
							: {};
					var __webpack_require___initializeSharingData_scopeToSharingDataMapping;
					const initializeSharingScopeToInitDataMapping =
						(__webpack_require___initializeSharingData_scopeToSharingDataMapping =
							(__webpack_require___initializeSharingData =
								__webpack_require__.initializeSharingData) === null ||
							__webpack_require___initializeSharingData === void 0
								? void 0
								: __webpack_require___initializeSharingData.scopeToSharingDataMapping) !==
							null &&
						__webpack_require___initializeSharingData_scopeToSharingDataMapping !==
							void 0
							? __webpack_require___initializeSharingData_scopeToSharingDataMapping
							: {};
					var __webpack_require___consumesLoadingData_chunkMapping;
					const consumesLoadingChunkMapping =
						(__webpack_require___consumesLoadingData_chunkMapping =
							(__webpack_require___consumesLoadingData =
								__webpack_require__.consumesLoadingData) === null ||
							__webpack_require___consumesLoadingData === void 0
								? void 0
								: __webpack_require___consumesLoadingData.chunkMapping) !==
							null &&
						__webpack_require___consumesLoadingData_chunkMapping !== void 0
							? __webpack_require___consumesLoadingData_chunkMapping
							: {};
					var __webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping;
					const consumesLoadingModuleToConsumeDataMapping =
						(__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping =
							(__webpack_require___consumesLoadingData1 =
								__webpack_require__.consumesLoadingData) === null ||
							__webpack_require___consumesLoadingData1 === void 0
								? void 0
								: __webpack_require___consumesLoadingData1.moduleIdToConsumeDataMapping) !==
							null &&
						__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping !==
							void 0
							? __webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping
							: {};
					const consumesLoadinginstalledModules = {};
					const initializeSharingInitPromises = [];
					const initializeSharingInitTokens = {};
					const containerShareScope =
						(__webpack_require___initializeExposesData =
							__webpack_require__.initializeExposesData) === null ||
						__webpack_require___initializeExposesData === void 0
							? void 0
							: __webpack_require___initializeExposesData.shareScope;
					for (const key in _Users_bytedance_RustroverProjects_rspack_node_modules_pnpm_module_federation_webpack_bundler_runtime_0_16_0_node_modules_module_federation_webpack_bundler_runtime_dist_index_cjs_cjs__WEBPACK_IMPORTED_MODULE_0___default()) {
						__webpack_require__.federation[key] =
							_Users_bytedance_RustroverProjects_rspack_node_modules_pnpm_module_federation_webpack_bundler_runtime_0_16_0_node_modules_module_federation_webpack_bundler_runtime_dist_index_cjs_cjs__WEBPACK_IMPORTED_MODULE_0___default()[
								key
							];
					}
					early(
						__webpack_require__.federation,
						"consumesLoadingModuleToHandlerMapping",
						() => {
							const consumesLoadingModuleToHandlerMapping = {};
							for (let [moduleId, data] of Object.entries(
								consumesLoadingModuleToConsumeDataMapping
							)) {
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
							return consumesLoadingModuleToHandlerMapping;
						}
					);
					early(__webpack_require__.federation, "initOptions", () => ({}));
					early(
						__webpack_require__.federation.initOptions,
						"name",
						() => __module_federation_container_name__
					);
					early(
						__webpack_require__.federation.initOptions,
						"shareStrategy",
						() => __module_federation_share_strategy__
					);
					early(__webpack_require__.federation.initOptions, "shared", () => {
						const shared = {};
						for (let [scope, stages] of Object.entries(
							initializeSharingScopeToInitDataMapping
						)) {
							for (let stage of stages) {
								if (typeof stage === "object" && stage !== null) {
									const {
										name,
										version,
										factory,
										eager,
										singleton,
										requiredVersion,
										strictVersion
									} = stage;
									const shareConfig = {};
									const isValidValue = function (val) {
										return typeof val !== "undefined";
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
										get: factory
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
					merge(__webpack_require__.federation.initOptions, "remotes", () =>
						Object.values(__module_federation_remote_infos__)
							.flat()
							.filter(remote => remote.externalType === "script")
					);
					merge(
						__webpack_require__.federation.initOptions,
						"plugins",
						() => __module_federation_runtime_plugins__
					);
					early(
						__webpack_require__.federation,
						"bundlerRuntimeOptions",
						() => ({})
					);
					early(
						__webpack_require__.federation.bundlerRuntimeOptions,
						"remotes",
						() => ({})
					);
					early(
						__webpack_require__.federation.bundlerRuntimeOptions.remotes,
						"chunkMapping",
						() => remotesLoadingChunkMapping
					);
					early(
						__webpack_require__.federation.bundlerRuntimeOptions.remotes,
						"idToExternalAndNameMapping",
						() => {
							const remotesLoadingIdToExternalAndNameMappingMapping = {};
							for (let [moduleId, data] of Object.entries(
								remotesLoadingModuleIdToRemoteDataMapping
							)) {
								remotesLoadingIdToExternalAndNameMappingMapping[moduleId] = [
									data.shareScope,
									data.name,
									data.externalModuleId,
									data.remoteName
								];
							}
							return remotesLoadingIdToExternalAndNameMappingMapping;
						}
					);
					early(
						__webpack_require__.federation.bundlerRuntimeOptions.remotes,
						"webpackRequire",
						() => __webpack_require__
					);
					merge(
						__webpack_require__.federation.bundlerRuntimeOptions.remotes,
						"idToRemoteMap",
						() => {
							const idToRemoteMap = {};
							for (let [id, remoteData] of Object.entries(
								remotesLoadingModuleIdToRemoteDataMapping
							)) {
								const info =
									__module_federation_remote_infos__[remoteData.remoteName];
								if (info) idToRemoteMap[id] = info;
							}
							return idToRemoteMap;
						}
					);
					override(
						__webpack_require__,
						"S",
						__webpack_require__.federation.bundlerRuntime.S
					);
					if (__webpack_require__.federation.attachShareScopeMap) {
						__webpack_require__.federation.attachShareScopeMap(
							__webpack_require__
						);
					}
					override(__webpack_require__.f, "remotes", (chunkId, promises) =>
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
							webpackRequire: __webpack_require__
						})
					);
					override(__webpack_require__.f, "consumes", (chunkId, promises) =>
						__webpack_require__.federation.bundlerRuntime.consumes({
							chunkId,
							promises,
							chunkMapping: consumesLoadingChunkMapping,
							moduleToHandlerMapping:
								__webpack_require__.federation
									.consumesLoadingModuleToHandlerMapping,
							installedModules: consumesLoadinginstalledModules,
							webpackRequire: __webpack_require__
						})
					);
					override(__webpack_require__, "I", (name, initScope) =>
						__webpack_require__.federation.bundlerRuntime.I({
							shareScopeName: name,
							initScope,
							initPromises: initializeSharingInitPromises,
							initTokens: initializeSharingInitTokens,
							webpackRequire: __webpack_require__
						})
					);
					override(
						__webpack_require__,
						"initContainer",
						(shareScope, initScope, remoteEntryInitOptions) =>
							__webpack_require__.federation.bundlerRuntime.initContainerEntry({
								shareScope,
								initScope,
								remoteEntryInitOptions,
								shareScopeKey: containerShareScope,
								webpackRequire: __webpack_require__
							})
					);
					override(__webpack_require__, "getContainer", (module1, getScope) => {
						var moduleMap = __webpack_require__.initializeExposesData.moduleMap;
						__webpack_require__.R = getScope;
						getScope = Object.prototype.hasOwnProperty.call(moduleMap, module1)
							? moduleMap[module1]()
							: Promise.resolve().then(() => {
									throw new Error(
										'Module "' + module1 + '" does not exist in container.'
									);
								});
						__webpack_require__.R = undefined;
						return getScope;
					});
					__webpack_require__.federation.instance =
						__webpack_require__.federation.runtime.init(
							__webpack_require__.federation.initOptions
						);
					if (
						(__webpack_require___consumesLoadingData2 =
							__webpack_require__.consumesLoadingData) === null ||
						__webpack_require___consumesLoadingData2 === void 0
							? void 0
							: __webpack_require___consumesLoadingData2.initialConsumes
					) {
						__webpack_require__.federation.bundlerRuntime.installInitialConsumes(
							{
								webpackRequire: __webpack_require__,
								installedModules: consumesLoadinginstalledModules,
								initialConsumes:
									__webpack_require__.consumesLoadingData.initialConsumes,
								moduleToHandlerMapping:
									__webpack_require__.federation
										.consumesLoadingModuleToHandlerMapping
							}
						);
					}
				}
			},
		"../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/index.cjs.cjs":
			/*!***************************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/index.cjs.cjs ***!
  \***************************************************************************************************************************************/
			function (__unused_webpack_module, exports, __webpack_require__) {
				var polyfills = __webpack_require__(
					/*! ./polyfills.cjs.cjs */ "../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/polyfills.cjs.cjs"
				);
				var sdk = __webpack_require__(
					/*! @module-federation/sdk */ "../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/index.cjs.cjs"
				);
				var errorCodes = __webpack_require__(
					/*! @module-federation/error-codes */ "../../node_modules/.pnpm/@module-federation+error-codes@0.16.0/node_modules/@module-federation/error-codes/dist/index.cjs.js"
				);

				const LOG_CATEGORY = "[ Federation Runtime ]";
				// FIXME: pre-bundle ?
				const logger = sdk.createLogger(LOG_CATEGORY);
				// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
				function assert(condition, msg) {
					if (!condition) {
						error(msg);
					}
				}
				function error(msg) {
					if (msg instanceof Error) {
						msg.message = `${LOG_CATEGORY}: ${msg.message}`;
						throw msg;
					}
					throw new Error(`${LOG_CATEGORY}: ${msg}`);
				}
				function warn(msg) {
					if (msg instanceof Error) {
						msg.message = `${LOG_CATEGORY}: ${msg.message}`;
						logger.warn(msg);
					} else {
						logger.warn(msg);
					}
				}

				function addUniqueItem(arr, item) {
					if (arr.findIndex(name => name === item) === -1) {
						arr.push(item);
					}
					return arr;
				}
				function getFMId(remoteInfo) {
					if ("version" in remoteInfo && remoteInfo.version) {
						return `${remoteInfo.name}:${remoteInfo.version}`;
					} else if ("entry" in remoteInfo && remoteInfo.entry) {
						return `${remoteInfo.name}:${remoteInfo.entry}`;
					} else {
						return `${remoteInfo.name}`;
					}
				}
				function isRemoteInfoWithEntry(remote) {
					return typeof remote.entry !== "undefined";
				}
				function isPureRemoteEntry(remote) {
					return !remote.entry.includes(".json");
				}
				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				async function safeWrapper(callback, disableWarn) {
					try {
						const res = await callback();
						return res;
					} catch (e) {
						!disableWarn && warn(e);
						return;
					}
				}
				function isObject(val) {
					return val && typeof val === "object";
				}
				const objectToString = Object.prototype.toString;
				// eslint-disable-next-line @typescript-eslint/ban-types
				function isPlainObject(val) {
					return objectToString.call(val) === "[object Object]";
				}
				function isStaticResourcesEqual(url1, url2) {
					const REG_EXP = /^(https?:)?\/\//i;
					// Transform url1 and url2 into relative paths
					const relativeUrl1 = url1.replace(REG_EXP, "").replace(/\/$/, "");
					const relativeUrl2 = url2.replace(REG_EXP, "").replace(/\/$/, "");
					// Check if the relative paths are identical
					return relativeUrl1 === relativeUrl2;
				}
				function arrayOptions(options) {
					return Array.isArray(options) ? options : [options];
				}
				function getRemoteEntryInfoFromSnapshot(snapshot) {
					const defaultRemoteEntryInfo = {
						url: "",
						type: "global",
						globalName: ""
					};
					if (sdk.isBrowserEnv() || sdk.isReactNativeEnv()) {
						return "remoteEntry" in snapshot
							? {
									url: snapshot.remoteEntry,
									type: snapshot.remoteEntryType,
									globalName: snapshot.globalName
								}
							: defaultRemoteEntryInfo;
					}
					if ("ssrRemoteEntry" in snapshot) {
						return {
							url: snapshot.ssrRemoteEntry || defaultRemoteEntryInfo.url,
							type: snapshot.ssrRemoteEntryType || defaultRemoteEntryInfo.type,
							globalName: snapshot.globalName
						};
					}
					return defaultRemoteEntryInfo;
				}
				const processModuleAlias = (name, subPath) => {
					// @host/ ./button -> @host/button
					let moduleName;
					if (name.endsWith("/")) {
						moduleName = name.slice(0, -1);
					} else {
						moduleName = name;
					}
					if (subPath.startsWith(".")) {
						subPath = subPath.slice(1);
					}
					moduleName = moduleName + subPath;
					return moduleName;
				};

				const CurrentGlobal =
					typeof globalThis === "object" ? globalThis : window;
				const nativeGlobal = (() => {
					try {
						// get real window (incase of sandbox)
						return document.defaultView;
					} catch (e) {
						// node env
						return CurrentGlobal;
					}
				})();
				const Global = nativeGlobal;
				function definePropertyGlobalVal(target, key, val) {
					Object.defineProperty(target, key, {
						value: val,
						configurable: false,
						writable: true
					});
				}
				function includeOwnProperty(target, key) {
					return Object.hasOwnProperty.call(target, key);
				}
				// This section is to prevent encapsulation by certain microfrontend frameworks. Due to reuse policies, sandbox escapes.
				// The sandbox in the microfrontend does not replicate the value of 'configurable'.
				// If there is no loading content on the global object, this section defines the loading object.
				if (
					!includeOwnProperty(CurrentGlobal, "__GLOBAL_LOADING_REMOTE_ENTRY__")
				) {
					definePropertyGlobalVal(
						CurrentGlobal,
						"__GLOBAL_LOADING_REMOTE_ENTRY__",
						{}
					);
				}
				const globalLoading = CurrentGlobal.__GLOBAL_LOADING_REMOTE_ENTRY__;
				function setGlobalDefaultVal(target) {
					var _target___FEDERATION__,
						_target___FEDERATION__1,
						_target___FEDERATION__2,
						_target___FEDERATION__3,
						_target___FEDERATION__4,
						_target___FEDERATION__5;
					if (
						includeOwnProperty(target, "__VMOK__") &&
						!includeOwnProperty(target, "__FEDERATION__")
					) {
						definePropertyGlobalVal(target, "__FEDERATION__", target.__VMOK__);
					}
					if (!includeOwnProperty(target, "__FEDERATION__")) {
						definePropertyGlobalVal(target, "__FEDERATION__", {
							__GLOBAL_PLUGIN__: [],
							__INSTANCES__: [],
							moduleInfo: {},
							__SHARE__: {},
							__MANIFEST_LOADING__: {},
							__PRELOADED_MAP__: new Map()
						});
						definePropertyGlobalVal(target, "__VMOK__", target.__FEDERATION__);
					}
					var ___GLOBAL_PLUGIN__;
					(___GLOBAL_PLUGIN__ = (_target___FEDERATION__ = target.__FEDERATION__)
						.__GLOBAL_PLUGIN__) != null
						? ___GLOBAL_PLUGIN__
						: (_target___FEDERATION__.__GLOBAL_PLUGIN__ = []);
					var ___INSTANCES__;
					(___INSTANCES__ = (_target___FEDERATION__1 = target.__FEDERATION__)
						.__INSTANCES__) != null
						? ___INSTANCES__
						: (_target___FEDERATION__1.__INSTANCES__ = []);
					var _moduleInfo;
					(_moduleInfo = (_target___FEDERATION__2 = target.__FEDERATION__)
						.moduleInfo) != null
						? _moduleInfo
						: (_target___FEDERATION__2.moduleInfo = {});
					var ___SHARE__;
					(___SHARE__ = (_target___FEDERATION__3 = target.__FEDERATION__)
						.__SHARE__) != null
						? ___SHARE__
						: (_target___FEDERATION__3.__SHARE__ = {});
					var ___MANIFEST_LOADING__;
					(___MANIFEST_LOADING__ = (_target___FEDERATION__4 =
						target.__FEDERATION__).__MANIFEST_LOADING__) != null
						? ___MANIFEST_LOADING__
						: (_target___FEDERATION__4.__MANIFEST_LOADING__ = {});
					var ___PRELOADED_MAP__;
					(___PRELOADED_MAP__ = (_target___FEDERATION__5 =
						target.__FEDERATION__).__PRELOADED_MAP__) != null
						? ___PRELOADED_MAP__
						: (_target___FEDERATION__5.__PRELOADED_MAP__ = new Map());
				}
				setGlobalDefaultVal(CurrentGlobal);
				setGlobalDefaultVal(nativeGlobal);
				function resetFederationGlobalInfo() {
					CurrentGlobal.__FEDERATION__.__GLOBAL_PLUGIN__ = [];
					CurrentGlobal.__FEDERATION__.__INSTANCES__ = [];
					CurrentGlobal.__FEDERATION__.moduleInfo = {};
					CurrentGlobal.__FEDERATION__.__SHARE__ = {};
					CurrentGlobal.__FEDERATION__.__MANIFEST_LOADING__ = {};
					Object.keys(globalLoading).forEach(key => {
						delete globalLoading[key];
					});
				}
				function setGlobalFederationInstance(FederationInstance) {
					CurrentGlobal.__FEDERATION__.__INSTANCES__.push(FederationInstance);
				}
				function getGlobalFederationConstructor() {
					return CurrentGlobal.__FEDERATION__.__DEBUG_CONSTRUCTOR__;
				}
				function setGlobalFederationConstructor(
					FederationConstructor,
					isDebug = sdk.isDebugMode()
				) {
					if (isDebug) {
						CurrentGlobal.__FEDERATION__.__DEBUG_CONSTRUCTOR__ =
							FederationConstructor;
						CurrentGlobal.__FEDERATION__.__DEBUG_CONSTRUCTOR_VERSION__ =
							"0.16.0";
					}
				}
				// eslint-disable-next-line @typescript-eslint/ban-types
				function getInfoWithoutType(target, key) {
					if (typeof key === "string") {
						const keyRes = target[key];
						if (keyRes) {
							return {
								value: target[key],
								key: key
							};
						} else {
							const targetKeys = Object.keys(target);
							for (const targetKey of targetKeys) {
								const [targetTypeOrName, _] = targetKey.split(":");
								const nKey = `${targetTypeOrName}:${key}`;
								const typeWithKeyRes = target[nKey];
								if (typeWithKeyRes) {
									return {
										value: typeWithKeyRes,
										key: nKey
									};
								}
							}
							return {
								value: undefined,
								key: key
							};
						}
					} else {
						throw new Error("key must be string");
					}
				}
				const getGlobalSnapshot = () => nativeGlobal.__FEDERATION__.moduleInfo;
				const getTargetSnapshotInfoByModuleInfo = (moduleInfo, snapshot) => {
					// Check if the remote is included in the hostSnapshot
					const moduleKey = getFMId(moduleInfo);
					const getModuleInfo = getInfoWithoutType(snapshot, moduleKey).value;
					// The remoteSnapshot might not include a version
					if (
						getModuleInfo &&
						!getModuleInfo.version &&
						"version" in moduleInfo &&
						moduleInfo["version"]
					) {
						getModuleInfo.version = moduleInfo["version"];
					}
					if (getModuleInfo) {
						return getModuleInfo;
					}
					// If the remote is not included in the hostSnapshot, deploy a micro app snapshot
					if ("version" in moduleInfo && moduleInfo["version"]) {
						const { version } = moduleInfo,
							resModuleInfo = polyfills._object_without_properties_loose(
								moduleInfo,
								["version"]
							);
						const moduleKeyWithoutVersion = getFMId(resModuleInfo);
						const getModuleInfoWithoutVersion = getInfoWithoutType(
							nativeGlobal.__FEDERATION__.moduleInfo,
							moduleKeyWithoutVersion
						).value;
						if (
							(getModuleInfoWithoutVersion == null
								? void 0
								: getModuleInfoWithoutVersion.version) === version
						) {
							return getModuleInfoWithoutVersion;
						}
					}
					return;
				};
				const getGlobalSnapshotInfoByModuleInfo = moduleInfo =>
					getTargetSnapshotInfoByModuleInfo(
						moduleInfo,
						nativeGlobal.__FEDERATION__.moduleInfo
					);
				const setGlobalSnapshotInfoByModuleInfo = (
					remoteInfo,
					moduleDetailInfo
				) => {
					const moduleKey = getFMId(remoteInfo);
					nativeGlobal.__FEDERATION__.moduleInfo[moduleKey] = moduleDetailInfo;
					return nativeGlobal.__FEDERATION__.moduleInfo;
				};
				const addGlobalSnapshot = moduleInfos => {
					nativeGlobal.__FEDERATION__.moduleInfo = polyfills._extends(
						{},
						nativeGlobal.__FEDERATION__.moduleInfo,
						moduleInfos
					);
					return () => {
						const keys = Object.keys(moduleInfos);
						for (const key of keys) {
							delete nativeGlobal.__FEDERATION__.moduleInfo[key];
						}
					};
				};
				const getRemoteEntryExports = (name, globalName) => {
					const remoteEntryKey = globalName || `__FEDERATION_${name}:custom__`;
					const entryExports = CurrentGlobal[remoteEntryKey];
					return {
						remoteEntryKey,
						entryExports
					};
				};
				// This function is used to register global plugins.
				// It iterates over the provided plugins and checks if they are already registered.
				// If a plugin is not registered, it is added to the global plugins.
				// If a plugin is already registered, a warning message is logged.
				const registerGlobalPlugins = plugins => {
					const { __GLOBAL_PLUGIN__ } = nativeGlobal.__FEDERATION__;
					plugins.forEach(plugin => {
						if (
							__GLOBAL_PLUGIN__.findIndex(p => p.name === plugin.name) === -1
						) {
							__GLOBAL_PLUGIN__.push(plugin);
						} else {
							warn(`The plugin ${plugin.name} has been registered.`);
						}
					});
				};
				const getGlobalHostPlugins = () =>
					nativeGlobal.__FEDERATION__.__GLOBAL_PLUGIN__;
				const getPreloaded = id =>
					CurrentGlobal.__FEDERATION__.__PRELOADED_MAP__.get(id);
				const setPreloaded = id =>
					CurrentGlobal.__FEDERATION__.__PRELOADED_MAP__.set(id, true);

				const DEFAULT_SCOPE = "default";
				const DEFAULT_REMOTE_TYPE = "global";

				// fork from https://github.com/originjs/vite-plugin-federation/blob/v1.1.12/packages/lib/src/utils/semver/index.ts
				// those constants are based on https://www.rubydoc.info/gems/semantic_range/3.0.0/SemanticRange#BUILDIDENTIFIER-constant
				// Copyright (c)
				// vite-plugin-federation is licensed under Mulan PSL v2.
				// You can use this software according to the terms and conditions of the Mulan PSL v2.
				// You may obtain a copy of Mulan PSL v2 at:
				//      http://license.coscl.org.cn/MulanPSL2
				// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
				// See the Mulan PSL v2 for more details.
				const buildIdentifier = "[0-9A-Za-z-]+";
				const build = `(?:\\+(${buildIdentifier}(?:\\.${buildIdentifier})*))`;
				const numericIdentifier = "0|[1-9]\\d*";
				const numericIdentifierLoose = "[0-9]+";
				const nonNumericIdentifier = "\\d*[a-zA-Z-][a-zA-Z0-9-]*";
				const preReleaseIdentifierLoose = `(?:${numericIdentifierLoose}|${nonNumericIdentifier})`;
				const preReleaseLoose = `(?:-?(${preReleaseIdentifierLoose}(?:\\.${preReleaseIdentifierLoose})*))`;
				const preReleaseIdentifier = `(?:${numericIdentifier}|${nonNumericIdentifier})`;
				const preRelease = `(?:-(${preReleaseIdentifier}(?:\\.${preReleaseIdentifier})*))`;
				const xRangeIdentifier = `${numericIdentifier}|x|X|\\*`;
				const xRangePlain = `[v=\\s]*(${xRangeIdentifier})(?:\\.(${xRangeIdentifier})(?:\\.(${xRangeIdentifier})(?:${preRelease})?${build}?)?)?`;
				const hyphenRange = `^\\s*(${xRangePlain})\\s+-\\s+(${xRangePlain})\\s*$`;
				const mainVersionLoose = `(${numericIdentifierLoose})\\.(${numericIdentifierLoose})\\.(${numericIdentifierLoose})`;
				const loosePlain = `[v=\\s]*${mainVersionLoose}${preReleaseLoose}?${build}?`;
				const gtlt = "((?:<|>)?=?)";
				const comparatorTrim = `(\\s*)${gtlt}\\s*(${loosePlain}|${xRangePlain})`;
				const loneTilde = "(?:~>?)";
				const tildeTrim = `(\\s*)${loneTilde}\\s+`;
				const loneCaret = "(?:\\^)";
				const caretTrim = `(\\s*)${loneCaret}\\s+`;
				const star = "(<|>)?=?\\s*\\*";
				const caret = `^${loneCaret}${xRangePlain}$`;
				const mainVersion = `(${numericIdentifier})\\.(${numericIdentifier})\\.(${numericIdentifier})`;
				const fullPlain = `v?${mainVersion}${preRelease}?${build}?`;
				const tilde = `^${loneTilde}${xRangePlain}$`;
				const xRange = `^${gtlt}\\s*${xRangePlain}$`;
				const comparator = `^${gtlt}\\s*(${fullPlain})$|^$`;
				// copy from semver package
				const gte0 = "^\\s*>=\\s*0.0.0\\s*$";

				// fork from https://github.com/originjs/vite-plugin-federation/blob/v1.1.12/packages/lib/src/utils/semver/index.ts
				// Copyright (c)
				// vite-plugin-federation is licensed under Mulan PSL v2.
				// You can use this software according to the terms and conditions of the Mulan PSL v2.
				// You may obtain a copy of Mulan PSL v2 at:
				//      http://license.coscl.org.cn/MulanPSL2
				// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
				// See the Mulan PSL v2 for more details.
				function parseRegex(source) {
					return new RegExp(source);
				}
				function isXVersion(version) {
					return !version || version.toLowerCase() === "x" || version === "*";
				}
				function pipe(...fns) {
					return x => fns.reduce((v, f) => f(v), x);
				}
				function extractComparator(comparatorString) {
					return comparatorString.match(parseRegex(comparator));
				}
				function combineVersion(major, minor, patch, preRelease) {
					const mainVersion = `${major}.${minor}.${patch}`;
					if (preRelease) {
						return `${mainVersion}-${preRelease}`;
					}
					return mainVersion;
				}

				// fork from https://github.com/originjs/vite-plugin-federation/blob/v1.1.12/packages/lib/src/utils/semver/index.ts
				// Copyright (c)
				// vite-plugin-federation is licensed under Mulan PSL v2.
				// You can use this software according to the terms and conditions of the Mulan PSL v2.
				// You may obtain a copy of Mulan PSL v2 at:
				//      http://license.coscl.org.cn/MulanPSL2
				// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
				// See the Mulan PSL v2 for more details.
				function parseHyphen(range) {
					return range.replace(
						parseRegex(hyphenRange),
						(
							_range,
							from,
							fromMajor,
							fromMinor,
							fromPatch,
							_fromPreRelease,
							_fromBuild,
							to,
							toMajor,
							toMinor,
							toPatch,
							toPreRelease
						) => {
							if (isXVersion(fromMajor)) {
								from = "";
							} else if (isXVersion(fromMinor)) {
								from = `>=${fromMajor}.0.0`;
							} else if (isXVersion(fromPatch)) {
								from = `>=${fromMajor}.${fromMinor}.0`;
							} else {
								from = `>=${from}`;
							}
							if (isXVersion(toMajor)) {
								to = "";
							} else if (isXVersion(toMinor)) {
								to = `<${Number(toMajor) + 1}.0.0-0`;
							} else if (isXVersion(toPatch)) {
								to = `<${toMajor}.${Number(toMinor) + 1}.0-0`;
							} else if (toPreRelease) {
								to = `<=${toMajor}.${toMinor}.${toPatch}-${toPreRelease}`;
							} else {
								to = `<=${to}`;
							}
							return `${from} ${to}`.trim();
						}
					);
				}
				function parseComparatorTrim(range) {
					return range.replace(parseRegex(comparatorTrim), "$1$2$3");
				}
				function parseTildeTrim(range) {
					return range.replace(parseRegex(tildeTrim), "$1~");
				}
				function parseCaretTrim(range) {
					return range.replace(parseRegex(caretTrim), "$1^");
				}
				function parseCarets(range) {
					return range
						.trim()
						.split(/\s+/)
						.map(rangeVersion =>
							rangeVersion.replace(
								parseRegex(caret),
								(_, major, minor, patch, preRelease) => {
									if (isXVersion(major)) {
										return "";
									} else if (isXVersion(minor)) {
										return `>=${major}.0.0 <${Number(major) + 1}.0.0-0`;
									} else if (isXVersion(patch)) {
										if (major === "0") {
											return `>=${major}.${minor}.0 <${major}.${Number(minor) + 1}.0-0`;
										} else {
											return `>=${major}.${minor}.0 <${Number(major) + 1}.0.0-0`;
										}
									} else if (preRelease) {
										if (major === "0") {
											if (minor === "0") {
												return `>=${major}.${minor}.${patch}-${preRelease} <${major}.${minor}.${Number(patch) + 1}-0`;
											} else {
												return `>=${major}.${minor}.${patch}-${preRelease} <${major}.${Number(minor) + 1}.0-0`;
											}
										} else {
											return `>=${major}.${minor}.${patch}-${preRelease} <${Number(major) + 1}.0.0-0`;
										}
									} else {
										if (major === "0") {
											if (minor === "0") {
												return `>=${major}.${minor}.${patch} <${major}.${minor}.${Number(patch) + 1}-0`;
											} else {
												return `>=${major}.${minor}.${patch} <${major}.${Number(minor) + 1}.0-0`;
											}
										}
										return `>=${major}.${minor}.${patch} <${Number(major) + 1}.0.0-0`;
									}
								}
							)
						)
						.join(" ");
				}
				function parseTildes(range) {
					return range
						.trim()
						.split(/\s+/)
						.map(rangeVersion =>
							rangeVersion.replace(
								parseRegex(tilde),
								(_, major, minor, patch, preRelease) => {
									if (isXVersion(major)) {
										return "";
									} else if (isXVersion(minor)) {
										return `>=${major}.0.0 <${Number(major) + 1}.0.0-0`;
									} else if (isXVersion(patch)) {
										return `>=${major}.${minor}.0 <${major}.${Number(minor) + 1}.0-0`;
									} else if (preRelease) {
										return `>=${major}.${minor}.${patch}-${preRelease} <${major}.${Number(minor) + 1}.0-0`;
									}
									return `>=${major}.${minor}.${patch} <${major}.${Number(minor) + 1}.0-0`;
								}
							)
						)
						.join(" ");
				}
				function parseXRanges(range) {
					return range
						.split(/\s+/)
						.map(rangeVersion =>
							rangeVersion
								.trim()
								.replace(
									parseRegex(xRange),
									(ret, gtlt, major, minor, patch, preRelease) => {
										const isXMajor = isXVersion(major);
										const isXMinor = isXMajor || isXVersion(minor);
										const isXPatch = isXMinor || isXVersion(patch);
										if (gtlt === "=" && isXPatch) {
											gtlt = "";
										}
										preRelease = "";
										if (isXMajor) {
											if (gtlt === ">" || gtlt === "<") {
												// nothing is allowed
												return "<0.0.0-0";
											} else {
												// nothing is forbidden
												return "*";
											}
										} else if (gtlt && isXPatch) {
											// replace X with 0
											if (isXMinor) {
												minor = 0;
											}
											patch = 0;
											if (gtlt === ">") {
												// >1 => >=2.0.0
												// >1.2 => >=1.3.0
												gtlt = ">=";
												if (isXMinor) {
													major = Number(major) + 1;
													minor = 0;
													patch = 0;
												} else {
													minor = Number(minor) + 1;
													patch = 0;
												}
											} else if (gtlt === "<=") {
												// <=0.7.x is actually <0.8.0, since any 0.7.x should pass
												// Similarly, <=7.x is actually <8.0.0, etc.
												gtlt = "<";
												if (isXMinor) {
													major = Number(major) + 1;
												} else {
													minor = Number(minor) + 1;
												}
											}
											if (gtlt === "<") {
												preRelease = "-0";
											}
											return `${gtlt + major}.${minor}.${patch}${preRelease}`;
										} else if (isXMinor) {
											return `>=${major}.0.0${preRelease} <${Number(major) + 1}.0.0-0`;
										} else if (isXPatch) {
											return `>=${major}.${minor}.0${preRelease} <${major}.${Number(minor) + 1}.0-0`;
										}
										return ret;
									}
								)
						)
						.join(" ");
				}
				function parseStar(range) {
					return range.trim().replace(parseRegex(star), "");
				}
				function parseGTE0(comparatorString) {
					return comparatorString.trim().replace(parseRegex(gte0), "");
				}

				// fork from https://github.com/originjs/vite-plugin-federation/blob/v1.1.12/packages/lib/src/utils/semver/index.ts
				// Copyright (c)
				// vite-plugin-federation is licensed under Mulan PSL v2.
				// You can use this software according to the terms and conditions of the Mulan PSL v2.
				// You may obtain a copy of Mulan PSL v2 at:
				//      http://license.coscl.org.cn/MulanPSL2
				// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
				// See the Mulan PSL v2 for more details.
				function compareAtom(rangeAtom, versionAtom) {
					rangeAtom = Number(rangeAtom) || rangeAtom;
					versionAtom = Number(versionAtom) || versionAtom;
					if (rangeAtom > versionAtom) {
						return 1;
					}
					if (rangeAtom === versionAtom) {
						return 0;
					}
					return -1;
				}
				function comparePreRelease(rangeAtom, versionAtom) {
					const { preRelease: rangePreRelease } = rangeAtom;
					const { preRelease: versionPreRelease } = versionAtom;
					if (rangePreRelease === undefined && Boolean(versionPreRelease)) {
						return 1;
					}
					if (Boolean(rangePreRelease) && versionPreRelease === undefined) {
						return -1;
					}
					if (
						rangePreRelease === undefined &&
						versionPreRelease === undefined
					) {
						return 0;
					}
					for (let i = 0, n = rangePreRelease.length; i <= n; i++) {
						const rangeElement = rangePreRelease[i];
						const versionElement = versionPreRelease[i];
						if (rangeElement === versionElement) {
							continue;
						}
						if (rangeElement === undefined && versionElement === undefined) {
							return 0;
						}
						if (!rangeElement) {
							return 1;
						}
						if (!versionElement) {
							return -1;
						}
						return compareAtom(rangeElement, versionElement);
					}
					return 0;
				}
				function compareVersion(rangeAtom, versionAtom) {
					return (
						compareAtom(rangeAtom.major, versionAtom.major) ||
						compareAtom(rangeAtom.minor, versionAtom.minor) ||
						compareAtom(rangeAtom.patch, versionAtom.patch) ||
						comparePreRelease(rangeAtom, versionAtom)
					);
				}
				function eq(rangeAtom, versionAtom) {
					return rangeAtom.version === versionAtom.version;
				}
				function compare(rangeAtom, versionAtom) {
					switch (rangeAtom.operator) {
						case "":
						case "=":
							return eq(rangeAtom, versionAtom);
						case ">":
							return compareVersion(rangeAtom, versionAtom) < 0;
						case ">=":
							return (
								eq(rangeAtom, versionAtom) ||
								compareVersion(rangeAtom, versionAtom) < 0
							);
						case "<":
							return compareVersion(rangeAtom, versionAtom) > 0;
						case "<=":
							return (
								eq(rangeAtom, versionAtom) ||
								compareVersion(rangeAtom, versionAtom) > 0
							);
						case undefined: {
							// mean * or x -> all versions
							return true;
						}
						default:
							return false;
					}
				}

				// fork from https://github.com/originjs/vite-plugin-federation/blob/v1.1.12/packages/lib/src/utils/semver/index.ts
				// Copyright (c)
				// vite-plugin-federation is licensed under Mulan PSL v2.
				// You can use this software according to the terms and conditions of the Mulan PSL v2.
				// You may obtain a copy of Mulan PSL v2 at:
				//      http://license.coscl.org.cn/MulanPSL2
				// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
				// See the Mulan PSL v2 for more details.
				function parseComparatorString(range) {
					return pipe(
						// handle caret
						// ^ --> * (any, kinda silly)
						// ^2, ^2.x, ^2.x.x --> >=2.0.0 <3.0.0-0
						// ^2.0, ^2.0.x --> >=2.0.0 <3.0.0-0
						// ^1.2, ^1.2.x --> >=1.2.0 <2.0.0-0
						// ^1.2.3 --> >=1.2.3 <2.0.0-0
						// ^1.2.0 --> >=1.2.0 <2.0.0-0
						parseCarets, // handle tilde
						// ~, ~> --> * (any, kinda silly)
						// ~2, ~2.x, ~2.x.x, ~>2, ~>2.x ~>2.x.x --> >=2.0.0 <3.0.0-0
						// ~2.0, ~2.0.x, ~>2.0, ~>2.0.x --> >=2.0.0 <2.1.0-0
						// ~1.2, ~1.2.x, ~>1.2, ~>1.2.x --> >=1.2.0 <1.3.0-0
						// ~1.2.3, ~>1.2.3 --> >=1.2.3 <1.3.0-0
						// ~1.2.0, ~>1.2.0 --> >=1.2.0 <1.3.0-0
						parseTildes,
						parseXRanges,
						parseStar
					)(range);
				}
				function parseRange(range) {
					return pipe(
						// handle hyphenRange
						// `1.2.3 - 1.2.4` => `>=1.2.3 <=1.2.4`
						parseHyphen, // handle trim comparator
						// `> 1.2.3 < 1.2.5` => `>1.2.3 <1.2.5`
						parseComparatorTrim, // handle trim tilde
						// `~ 1.2.3` => `~1.2.3`
						parseTildeTrim, // handle trim caret
						// `^ 1.2.3` => `^1.2.3`
						parseCaretTrim
					)(range.trim())
						.split(/\s+/)
						.join(" ");
				}
				function satisfy(version, range) {
					if (!version) {
						return false;
					}
					// Extract version details once
					const extractedVersion = extractComparator(version);
					if (!extractedVersion) {
						// If the version string is invalid, it can't satisfy any range
						return false;
					}
					const [
						,
						versionOperator,
						,
						versionMajor,
						versionMinor,
						versionPatch,
						versionPreRelease
					] = extractedVersion;
					const versionAtom = {
						operator: versionOperator,
						version: combineVersion(
							versionMajor,
							versionMinor,
							versionPatch,
							versionPreRelease
						),
						major: versionMajor,
						minor: versionMinor,
						patch: versionPatch,
						preRelease:
							versionPreRelease == null ? void 0 : versionPreRelease.split(".")
					};
					// Split the range by || to handle OR conditions
					const orRanges = range.split("||");
					for (const orRange of orRanges) {
						const trimmedOrRange = orRange.trim();
						if (!trimmedOrRange) {
							// An empty range string signifies wildcard *, satisfy any valid version
							// (We already checked if the version itself is valid)
							return true;
						}
						// Handle simple wildcards explicitly before complex parsing
						if (trimmedOrRange === "*" || trimmedOrRange === "x") {
							return true;
						}
						try {
							// Apply existing parsing logic to the current OR sub-range
							const parsedSubRange = parseRange(trimmedOrRange); // Handles hyphens, trims etc.
							// Check if the result of initial parsing is empty, which can happen
							// for some wildcard cases handled by parseRange/parseComparatorString.
							// E.g. `parseStar` used in `parseComparatorString` returns ''.
							if (!parsedSubRange.trim()) {
								// If parsing results in empty string, treat as wildcard match
								return true;
							}
							const parsedComparatorString = parsedSubRange
								.split(" ")
								.map(rangeVersion => parseComparatorString(rangeVersion)) // Expands ^, ~
								.join(" ");
							// Check again if the comparator string became empty after specific parsing like ^ or ~
							if (!parsedComparatorString.trim()) {
								return true;
							}
							// Split the sub-range by space for implicit AND conditions
							const comparators = parsedComparatorString
								.split(/\s+/)
								.map(comparator => parseGTE0(comparator)) // Filter out empty strings that might result from multiple spaces
								.filter(Boolean);
							// If a sub-range becomes empty after parsing (e.g., invalid characters),
							// it cannot be satisfied. This check might be redundant now but kept for safety.
							if (comparators.length === 0) {
								continue;
							}
							let subRangeSatisfied = true;
							for (const comparator of comparators) {
								const extractedComparator = extractComparator(comparator);
								// If any part of the AND sub-range is invalid, the sub-range is not satisfied
								if (!extractedComparator) {
									subRangeSatisfied = false;
									break;
								}
								const [
									,
									rangeOperator,
									,
									rangeMajor,
									rangeMinor,
									rangePatch,
									rangePreRelease
								] = extractedComparator;
								const rangeAtom = {
									operator: rangeOperator,
									version: combineVersion(
										rangeMajor,
										rangeMinor,
										rangePatch,
										rangePreRelease
									),
									major: rangeMajor,
									minor: rangeMinor,
									patch: rangePatch,
									preRelease:
										rangePreRelease == null
											? void 0
											: rangePreRelease.split(".")
								};
								// Check if the version satisfies this specific comparator in the AND chain
								if (!compare(rangeAtom, versionAtom)) {
									subRangeSatisfied = false; // This part of the AND condition failed
									break; // No need to check further comparators in this sub-range
								}
							}
							// If all AND conditions within this OR sub-range were met, the overall range is satisfied
							if (subRangeSatisfied) {
								return true;
							}
						} catch (e) {
							// Log error and treat this sub-range as unsatisfied
							console.error(
								`[semver] Error processing range part "${trimmedOrRange}":`,
								e
							);
							continue;
						}
					}
					// If none of the OR sub-ranges were satisfied
					return false;
				}

				function formatShare(shareArgs, from, name, shareStrategy) {
					let get;
					if ("get" in shareArgs) {
						// eslint-disable-next-line prefer-destructuring
						get = shareArgs.get;
					} else if ("lib" in shareArgs) {
						get = () => Promise.resolve(shareArgs.lib);
					} else {
						get = () =>
							Promise.resolve(() => {
								throw new Error(`Can not get shared '${name}'!`);
							});
					}
					var _shareArgs_version, _shareArgs_scope, _shareArgs_strategy;
					return polyfills._extends(
						{
							deps: [],
							useIn: [],
							from,
							loading: null
						},
						shareArgs,
						{
							shareConfig: polyfills._extends(
								{
									requiredVersion: `^${shareArgs.version}`,
									singleton: false,
									eager: false,
									strictVersion: false
								},
								shareArgs.shareConfig
							),
							get,
							loaded:
								(shareArgs == null ? void 0 : shareArgs.loaded) ||
								"lib" in shareArgs
									? true
									: undefined,
							version:
								(_shareArgs_version = shareArgs.version) != null
									? _shareArgs_version
									: "0",
							scope: Array.isArray(shareArgs.scope)
								? shareArgs.scope
								: [
										(_shareArgs_scope = shareArgs.scope) != null
											? _shareArgs_scope
											: "default"
									],
							strategy:
								((_shareArgs_strategy = shareArgs.strategy) != null
									? _shareArgs_strategy
									: shareStrategy) || "version-first"
						}
					);
				}
				function formatShareConfigs(globalOptions, userOptions) {
					const shareArgs = userOptions.shared || {};
					const from = userOptions.name;
					const shareInfos = Object.keys(shareArgs).reduce((res, pkgName) => {
						const arrayShareArgs = arrayOptions(shareArgs[pkgName]);
						res[pkgName] = res[pkgName] || [];
						arrayShareArgs.forEach(shareConfig => {
							res[pkgName].push(
								formatShare(
									shareConfig,
									from,
									pkgName,
									userOptions.shareStrategy
								)
							);
						});
						return res;
					}, {});
					const shared = polyfills._extends({}, globalOptions.shared);
					Object.keys(shareInfos).forEach(shareKey => {
						if (!shared[shareKey]) {
							shared[shareKey] = shareInfos[shareKey];
						} else {
							shareInfos[shareKey].forEach(newUserSharedOptions => {
								const isSameVersion = shared[shareKey].find(
									sharedVal =>
										sharedVal.version === newUserSharedOptions.version
								);
								if (!isSameVersion) {
									shared[shareKey].push(newUserSharedOptions);
								}
							});
						}
					});
					return {
						shared,
						shareInfos
					};
				}
				function versionLt(a, b) {
					const transformInvalidVersion = version => {
						const isNumberVersion = !Number.isNaN(Number(version));
						if (isNumberVersion) {
							const splitArr = version.split(".");
							let validVersion = version;
							for (let i = 0; i < 3 - splitArr.length; i++) {
								validVersion += ".0";
							}
							return validVersion;
						}
						return version;
					};
					if (
						satisfy(
							transformInvalidVersion(a),
							`<=${transformInvalidVersion(b)}`
						)
					) {
						return true;
					} else {
						return false;
					}
				}
				const findVersion = (shareVersionMap, cb) => {
					const callback =
						cb ||
						function (prev, cur) {
							return versionLt(prev, cur);
						};
					return Object.keys(shareVersionMap).reduce((prev, cur) => {
						if (!prev) {
							return cur;
						}
						if (callback(prev, cur)) {
							return cur;
						}
						// default version is '0' https://github.com/webpack/webpack/blob/main/lib/sharing/ProvideSharedModule.js#L136
						if (prev === "0") {
							return cur;
						}
						return prev;
					}, 0);
				};
				const isLoaded = shared => {
					return Boolean(shared.loaded) || typeof shared.lib === "function";
				};
				const isLoading = shared => {
					return Boolean(shared.loading);
				};
				function findSingletonVersionOrderByVersion(
					shareScopeMap,
					scope,
					pkgName
				) {
					const versions = shareScopeMap[scope][pkgName];
					const callback = function (prev, cur) {
						return !isLoaded(versions[prev]) && versionLt(prev, cur);
					};
					return findVersion(shareScopeMap[scope][pkgName], callback);
				}
				function findSingletonVersionOrderByLoaded(
					shareScopeMap,
					scope,
					pkgName
				) {
					const versions = shareScopeMap[scope][pkgName];
					const callback = function (prev, cur) {
						const isLoadingOrLoaded = shared => {
							return isLoaded(shared) || isLoading(shared);
						};
						if (isLoadingOrLoaded(versions[cur])) {
							if (isLoadingOrLoaded(versions[prev])) {
								return Boolean(versionLt(prev, cur));
							} else {
								return true;
							}
						}
						if (isLoadingOrLoaded(versions[prev])) {
							return false;
						}
						return versionLt(prev, cur);
					};
					return findVersion(shareScopeMap[scope][pkgName], callback);
				}
				function getFindShareFunction(strategy) {
					if (strategy === "loaded-first") {
						return findSingletonVersionOrderByLoaded;
					}
					return findSingletonVersionOrderByVersion;
				}
				function getRegisteredShare(
					localShareScopeMap,
					pkgName,
					shareInfo,
					resolveShare
				) {
					if (!localShareScopeMap) {
						return;
					}
					const { shareConfig, scope = DEFAULT_SCOPE, strategy } = shareInfo;
					const scopes = Array.isArray(scope) ? scope : [scope];
					for (const sc of scopes) {
						if (
							shareConfig &&
							localShareScopeMap[sc] &&
							localShareScopeMap[sc][pkgName]
						) {
							const { requiredVersion } = shareConfig;
							const findShareFunction = getFindShareFunction(strategy);
							const maxOrSingletonVersion = findShareFunction(
								localShareScopeMap,
								sc,
								pkgName
							);
							//@ts-ignore
							const defaultResolver = () => {
								if (shareConfig.singleton) {
									if (
										typeof requiredVersion === "string" &&
										!satisfy(maxOrSingletonVersion, requiredVersion)
									) {
										const msg = `Version ${maxOrSingletonVersion} from ${maxOrSingletonVersion && localShareScopeMap[sc][pkgName][maxOrSingletonVersion].from} of shared singleton module ${pkgName} does not satisfy the requirement of ${shareInfo.from} which needs ${requiredVersion})`;
										if (shareConfig.strictVersion) {
											error(msg);
										} else {
											warn(msg);
										}
									}
									return localShareScopeMap[sc][pkgName][maxOrSingletonVersion];
								} else {
									if (requiredVersion === false || requiredVersion === "*") {
										return localShareScopeMap[sc][pkgName][
											maxOrSingletonVersion
										];
									}
									if (satisfy(maxOrSingletonVersion, requiredVersion)) {
										return localShareScopeMap[sc][pkgName][
											maxOrSingletonVersion
										];
									}
									for (const [versionKey, versionValue] of Object.entries(
										localShareScopeMap[sc][pkgName]
									)) {
										if (satisfy(versionKey, requiredVersion)) {
											return versionValue;
										}
									}
								}
							};
							const params = {
								shareScopeMap: localShareScopeMap,
								scope: sc,
								pkgName,
								version: maxOrSingletonVersion,
								GlobalFederation: Global.__FEDERATION__,
								resolver: defaultResolver
							};
							const resolveShared = resolveShare.emit(params) || params;
							return resolveShared.resolver();
						}
					}
				}
				function getGlobalShareScope() {
					return Global.__FEDERATION__.__SHARE__;
				}
				function getTargetSharedOptions(options) {
					const { pkgName, extraOptions, shareInfos } = options;
					const defaultResolver = sharedOptions => {
						if (!sharedOptions) {
							return undefined;
						}
						const shareVersionMap = {};
						sharedOptions.forEach(shared => {
							shareVersionMap[shared.version] = shared;
						});
						const callback = function (prev, cur) {
							return !isLoaded(shareVersionMap[prev]) && versionLt(prev, cur);
						};
						const maxVersion = findVersion(shareVersionMap, callback);
						return shareVersionMap[maxVersion];
					};
					var _extraOptions_resolver;
					const resolver =
						(_extraOptions_resolver =
							extraOptions == null ? void 0 : extraOptions.resolver) != null
							? _extraOptions_resolver
							: defaultResolver;
					return Object.assign(
						{},
						resolver(shareInfos[pkgName]),
						extraOptions == null ? void 0 : extraOptions.customShareInfo
					);
				}

				const ShareUtils = {
					getRegisteredShare,
					getGlobalShareScope
				};
				const GlobalUtils = {
					Global,
					nativeGlobal,
					resetFederationGlobalInfo,
					setGlobalFederationInstance,
					getGlobalFederationConstructor,
					setGlobalFederationConstructor,
					getInfoWithoutType,
					getGlobalSnapshot,
					getTargetSnapshotInfoByModuleInfo,
					getGlobalSnapshotInfoByModuleInfo,
					setGlobalSnapshotInfoByModuleInfo,
					addGlobalSnapshot,
					getRemoteEntryExports,
					registerGlobalPlugins,
					getGlobalHostPlugins,
					getPreloaded,
					setPreloaded
				};
				var helpers = {
					global: GlobalUtils,
					share: ShareUtils
				};

				function getBuilderId() {
					//@ts-ignore
					return typeof FEDERATION_BUILD_IDENTIFIER !== "undefined"
						? FEDERATION_BUILD_IDENTIFIER
						: "";
				}

				// Function to match a remote with its name and expose
				// id: pkgName(@federation/app1) + expose(button) = @federation/app1/button
				// id: alias(app1) + expose(button) = app1/button
				// id: alias(app1/utils) + expose(loadash/sort) = app1/utils/loadash/sort
				function matchRemoteWithNameAndExpose(remotes, id) {
					for (const remote of remotes) {
						// match pkgName
						const isNameMatched = id.startsWith(remote.name);
						let expose = id.replace(remote.name, "");
						if (isNameMatched) {
							if (expose.startsWith("/")) {
								const pkgNameOrAlias = remote.name;
								expose = `.${expose}`;
								return {
									pkgNameOrAlias,
									expose,
									remote
								};
							} else if (expose === "") {
								return {
									pkgNameOrAlias: remote.name,
									expose: ".",
									remote
								};
							}
						}
						// match alias
						const isAliasMatched = remote.alias && id.startsWith(remote.alias);
						let exposeWithAlias = remote.alias && id.replace(remote.alias, "");
						if (remote.alias && isAliasMatched) {
							if (exposeWithAlias && exposeWithAlias.startsWith("/")) {
								const pkgNameOrAlias = remote.alias;
								exposeWithAlias = `.${exposeWithAlias}`;
								return {
									pkgNameOrAlias,
									expose: exposeWithAlias,
									remote
								};
							} else if (exposeWithAlias === "") {
								return {
									pkgNameOrAlias: remote.alias,
									expose: ".",
									remote
								};
							}
						}
					}
					return;
				}
				// Function to match a remote with its name or alias
				function matchRemote(remotes, nameOrAlias) {
					for (const remote of remotes) {
						const isNameMatched = nameOrAlias === remote.name;
						if (isNameMatched) {
							return remote;
						}
						const isAliasMatched = remote.alias && nameOrAlias === remote.alias;
						if (isAliasMatched) {
							return remote;
						}
					}
					return;
				}

				function registerPlugins(plugins, hookInstances) {
					const globalPlugins = getGlobalHostPlugins();
					// Incorporate global plugins
					if (globalPlugins.length > 0) {
						globalPlugins.forEach(plugin => {
							if (
								plugins == null
									? void 0
									: plugins.find(item => item.name !== plugin.name)
							) {
								plugins.push(plugin);
							}
						});
					}
					if (plugins && plugins.length > 0) {
						plugins.forEach(plugin => {
							hookInstances.forEach(hookInstance => {
								hookInstance.applyPlugin(plugin);
							});
						});
					}
					return plugins;
				}

				const importCallback = ".then(callbacks[0]).catch(callbacks[1])";
				async function loadEsmEntry({ entry, remoteEntryExports }) {
					return new Promise((resolve, reject) => {
						try {
							if (!remoteEntryExports) {
								if (typeof FEDERATION_ALLOW_NEW_FUNCTION !== "undefined") {
									new Function(
										"callbacks",
										`import("${entry}")${importCallback}`
									)([resolve, reject]);
								} else {
									import(/* webpackIgnore: true */ /* @vite-ignore */ entry)
										.then(resolve)
										.catch(reject);
								}
							} else {
								resolve(remoteEntryExports);
							}
						} catch (e) {
							reject(e);
						}
					});
				}
				async function loadSystemJsEntry({ entry, remoteEntryExports }) {
					return new Promise((resolve, reject) => {
						try {
							if (!remoteEntryExports) {
								//@ts-ignore
								if (false) {
								} else {
									new Function(
										"callbacks",
										`System.import("${entry}")${importCallback}`
									)([resolve, reject]);
								}
							} else {
								resolve(remoteEntryExports);
							}
						} catch (e) {
							reject(e);
						}
					});
				}
				function handleRemoteEntryLoaded(name, globalName, entry) {
					const { remoteEntryKey, entryExports } = getRemoteEntryExports(
						name,
						globalName
					);
					assert(
						entryExports,
						errorCodes.getShortErrorMsg(
							errorCodes.RUNTIME_001,
							errorCodes.runtimeDescMap,
							{
								remoteName: name,
								remoteEntryUrl: entry,
								remoteEntryKey
							}
						)
					);
					return entryExports;
				}
				async function loadEntryScript({
					name,
					globalName,
					entry,
					loaderHook
				}) {
					const { entryExports: remoteEntryExports } = getRemoteEntryExports(
						name,
						globalName
					);
					if (remoteEntryExports) {
						return remoteEntryExports;
					}
					return sdk
						.loadScript(entry, {
							attrs: {},
							createScriptHook: (url, attrs) => {
								const res = loaderHook.lifecycle.createScript.emit({
									url,
									attrs
								});
								if (!res) return;
								if (res instanceof HTMLScriptElement) {
									return res;
								}
								if ("script" in res || "timeout" in res) {
									return res;
								}
								return;
							}
						})
						.then(() => {
							return handleRemoteEntryLoaded(name, globalName, entry);
						})
						.catch(e => {
							assert(
								undefined,
								errorCodes.getShortErrorMsg(
									errorCodes.RUNTIME_008,
									errorCodes.runtimeDescMap,
									{
										remoteName: name,
										resourceUrl: entry
									}
								)
							);
							throw e;
						});
				}
				async function loadEntryDom({
					remoteInfo,
					remoteEntryExports,
					loaderHook
				}) {
					const { entry, entryGlobalName: globalName, name, type } = remoteInfo;
					switch (type) {
						case "esm":
						case "module":
							return loadEsmEntry({
								entry,
								remoteEntryExports
							});
						case "system":
							return loadSystemJsEntry({
								entry,
								remoteEntryExports
							});
						default:
							return loadEntryScript({
								entry,
								globalName,
								name,
								loaderHook
							});
					}
				}
				async function loadEntryNode({ remoteInfo, loaderHook }) {
					const { entry, entryGlobalName: globalName, name, type } = remoteInfo;
					const { entryExports: remoteEntryExports } = getRemoteEntryExports(
						name,
						globalName
					);
					if (remoteEntryExports) {
						return remoteEntryExports;
					}
					return sdk
						.loadScriptNode(entry, {
							attrs: {
								name,
								globalName,
								type
							},
							loaderHook: {
								createScriptHook: (url, attrs = {}) => {
									const res = loaderHook.lifecycle.createScript.emit({
										url,
										attrs
									});
									if (!res) return;
									if ("url" in res) {
										return res;
									}
									return;
								}
							}
						})
						.then(() => {
							return handleRemoteEntryLoaded(name, globalName, entry);
						})
						.catch(e => {
							throw e;
						});
				}
				function getRemoteEntryUniqueKey(remoteInfo) {
					const { entry, name } = remoteInfo;
					return sdk.composeKeyWithSeparator(name, entry);
				}
				async function getRemoteEntry({
					origin,
					remoteEntryExports,
					remoteInfo
				}) {
					const uniqueKey = getRemoteEntryUniqueKey(remoteInfo);
					if (remoteEntryExports) {
						return remoteEntryExports;
					}
					if (!globalLoading[uniqueKey]) {
						const loadEntryHook =
							origin.remoteHandler.hooks.lifecycle.loadEntry;
						const loaderHook = origin.loaderHook;
						globalLoading[uniqueKey] = loadEntryHook
							.emit({
								loaderHook,
								remoteInfo,
								remoteEntryExports
							})
							.then(res => {
								if (res) {
									return res;
								}
								// Use ENV_TARGET if defined, otherwise fallback to isBrowserEnv, must keep this
								const isWebEnvironment =
									typeof ENV_TARGET !== "undefined"
										? ENV_TARGET === "web"
										: sdk.isBrowserEnv();
								return isWebEnvironment
									? loadEntryDom({
											remoteInfo,
											remoteEntryExports,
											loaderHook
										})
									: loadEntryNode({
											remoteInfo,
											loaderHook
										});
							});
					}
					return globalLoading[uniqueKey];
				}
				function getRemoteInfo(remote) {
					return polyfills._extends({}, remote, {
						entry: "entry" in remote ? remote.entry : "",
						type: remote.type || DEFAULT_REMOTE_TYPE,
						entryGlobalName: remote.entryGlobalName || remote.name,
						shareScope: remote.shareScope || DEFAULT_SCOPE
					});
				}

				let Module = class Module {
					async getEntry() {
						if (this.remoteEntryExports) {
							return this.remoteEntryExports;
						}
						let remoteEntryExports;
						try {
							remoteEntryExports = await getRemoteEntry({
								origin: this.host,
								remoteInfo: this.remoteInfo,
								remoteEntryExports: this.remoteEntryExports
							});
						} catch (err) {
							const uniqueKey = getRemoteEntryUniqueKey(this.remoteInfo);
							remoteEntryExports =
								await this.host.loaderHook.lifecycle.loadEntryError.emit({
									getRemoteEntry,
									origin: this.host,
									remoteInfo: this.remoteInfo,
									remoteEntryExports: this.remoteEntryExports,
									globalLoading,
									uniqueKey
								});
						}
						assert(
							remoteEntryExports,
							`remoteEntryExports is undefined \n ${sdk.safeToString(this.remoteInfo)}`
						);
						this.remoteEntryExports = remoteEntryExports;
						return this.remoteEntryExports;
					}
					// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
					async get(id, expose, options, remoteSnapshot) {
						const { loadFactory = true } = options || {
							loadFactory: true
						};
						// Get remoteEntry.js
						const remoteEntryExports = await this.getEntry();
						if (!this.inited) {
							const localShareScopeMap = this.host.shareScopeMap;
							const shareScopeKeys = Array.isArray(this.remoteInfo.shareScope)
								? this.remoteInfo.shareScope
								: [this.remoteInfo.shareScope];
							if (!shareScopeKeys.length) {
								shareScopeKeys.push("default");
							}
							shareScopeKeys.forEach(shareScopeKey => {
								if (!localShareScopeMap[shareScopeKey]) {
									localShareScopeMap[shareScopeKey] = {};
								}
							});
							// TODO: compate legacy init params, should use shareScopeMap if exist
							const shareScope = localShareScopeMap[shareScopeKeys[0]];
							const initScope = [];
							const remoteEntryInitOptions = {
								version: this.remoteInfo.version || "",
								shareScopeKeys: Array.isArray(this.remoteInfo.shareScope)
									? shareScopeKeys
									: this.remoteInfo.shareScope || "default"
							};
							// Help to find host instance
							Object.defineProperty(remoteEntryInitOptions, "shareScopeMap", {
								value: localShareScopeMap,
								// remoteEntryInitOptions will be traversed and assigned during container init, ,so this attribute is not allowed to be traversed
								enumerable: false
							});
							const initContainerOptions =
								await this.host.hooks.lifecycle.beforeInitContainer.emit({
									shareScope,
									// @ts-ignore shareScopeMap will be set by Object.defineProperty
									remoteEntryInitOptions,
									initScope,
									remoteInfo: this.remoteInfo,
									origin: this.host
								});
							if (
								typeof (remoteEntryExports == null
									? void 0
									: remoteEntryExports.init) === "undefined"
							) {
								error(
									errorCodes.getShortErrorMsg(
										errorCodes.RUNTIME_002,
										errorCodes.runtimeDescMap,
										{
											hostName: this.host.name,
											remoteName: this.remoteInfo.name,
											remoteEntryUrl: this.remoteInfo.entry,
											remoteEntryKey: this.remoteInfo.entryGlobalName
										}
									)
								);
							}
							await remoteEntryExports.init(
								initContainerOptions.shareScope,
								initContainerOptions.initScope,
								initContainerOptions.remoteEntryInitOptions
							);
							await this.host.hooks.lifecycle.initContainer.emit(
								polyfills._extends({}, initContainerOptions, {
									id,
									remoteSnapshot,
									remoteEntryExports
								})
							);
						}
						this.lib = remoteEntryExports;
						this.inited = true;
						let moduleFactory;
						moduleFactory =
							await this.host.loaderHook.lifecycle.getModuleFactory.emit({
								remoteEntryExports,
								expose,
								moduleInfo: this.remoteInfo
							});
						// get exposeGetter
						if (!moduleFactory) {
							moduleFactory = await remoteEntryExports.get(expose);
						}
						assert(
							moduleFactory,
							`${getFMId(this.remoteInfo)} remote don't export ${expose}.`
						);
						// keep symbol for module name always one format
						const symbolName = processModuleAlias(this.remoteInfo.name, expose);
						const wrapModuleFactory = this.wraperFactory(
							moduleFactory,
							symbolName
						);
						if (!loadFactory) {
							return wrapModuleFactory;
						}
						const exposeContent = await wrapModuleFactory();
						return exposeContent;
					}
					wraperFactory(moduleFactory, id) {
						function defineModuleId(res, id) {
							if (
								res &&
								typeof res === "object" &&
								Object.isExtensible(res) &&
								!Object.getOwnPropertyDescriptor(
									res,
									Symbol.for("mf_module_id")
								)
							) {
								Object.defineProperty(res, Symbol.for("mf_module_id"), {
									value: id,
									enumerable: false
								});
							}
						}
						if (moduleFactory instanceof Promise) {
							return async () => {
								const res = await moduleFactory();
								// This parameter is used for bridge debugging
								defineModuleId(res, id);
								return res;
							};
						} else {
							return () => {
								const res = moduleFactory();
								// This parameter is used for bridge debugging
								defineModuleId(res, id);
								return res;
							};
						}
					}
					constructor({ remoteInfo, host }) {
						this.inited = false;
						this.lib = undefined;
						this.remoteInfo = remoteInfo;
						this.host = host;
					}
				};

				class SyncHook {
					on(fn) {
						if (typeof fn === "function") {
							this.listeners.add(fn);
						}
					}
					once(fn) {
						// eslint-disable-next-line @typescript-eslint/no-this-alias
						const self = this;
						this.on(function wrapper(...args) {
							self.remove(wrapper);
							// eslint-disable-next-line prefer-spread
							return fn.apply(null, args);
						});
					}
					emit(...data) {
						let result;
						if (this.listeners.size > 0) {
							// eslint-disable-next-line prefer-spread
							this.listeners.forEach(fn => {
								result = fn(...data);
							});
						}
						return result;
					}
					remove(fn) {
						this.listeners.delete(fn);
					}
					removeAll() {
						this.listeners.clear();
					}
					constructor(type) {
						this.type = "";
						this.listeners = new Set();
						if (type) {
							this.type = type;
						}
					}
				}

				class AsyncHook extends SyncHook {
					emit(...data) {
						let result;
						const ls = Array.from(this.listeners);
						if (ls.length > 0) {
							let i = 0;
							const call = prev => {
								if (prev === false) {
									return false; // Abort process
								} else if (i < ls.length) {
									return Promise.resolve(ls[i++].apply(null, data)).then(call);
								} else {
									return prev;
								}
							};
							result = call();
						}
						return Promise.resolve(result);
					}
				}

				// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
				function checkReturnData(originalData, returnedData) {
					if (!isObject(returnedData)) {
						return false;
					}
					if (originalData !== returnedData) {
						// eslint-disable-next-line no-restricted-syntax
						for (const key in originalData) {
							if (!(key in returnedData)) {
								return false;
							}
						}
					}
					return true;
				}
				class SyncWaterfallHook extends SyncHook {
					emit(data) {
						if (!isObject(data)) {
							error(
								`The data for the "${this.type}" hook should be an object.`
							);
						}
						for (const fn of this.listeners) {
							try {
								const tempData = fn(data);
								if (checkReturnData(data, tempData)) {
									data = tempData;
								} else {
									this.onerror(
										`A plugin returned an unacceptable value for the "${this.type}" type.`
									);
									break;
								}
							} catch (e) {
								warn(e);
								this.onerror(e);
							}
						}
						return data;
					}
					constructor(type) {
						(super(), (this.onerror = error));
						this.type = type;
					}
				}

				class AsyncWaterfallHook extends SyncHook {
					emit(data) {
						if (!isObject(data)) {
							error(
								`The response data for the "${this.type}" hook must be an object.`
							);
						}
						const ls = Array.from(this.listeners);
						if (ls.length > 0) {
							let i = 0;
							const processError = e => {
								warn(e);
								this.onerror(e);
								return data;
							};
							const call = prevData => {
								if (checkReturnData(data, prevData)) {
									data = prevData;
									if (i < ls.length) {
										try {
											return Promise.resolve(ls[i++](data)).then(
												call,
												processError
											);
										} catch (e) {
											return processError(e);
										}
									}
								} else {
									this.onerror(
										`A plugin returned an incorrect value for the "${this.type}" type.`
									);
								}
								return data;
							};
							return Promise.resolve(call(data));
						}
						return Promise.resolve(data);
					}
					constructor(type) {
						(super(), (this.onerror = error));
						this.type = type;
					}
				}

				class PluginSystem {
					applyPlugin(plugin) {
						assert(isPlainObject(plugin), "Plugin configuration is invalid.");
						// The plugin's name is mandatory and must be unique
						const pluginName = plugin.name;
						assert(pluginName, "A name must be provided by the plugin.");
						if (!this.registerPlugins[pluginName]) {
							this.registerPlugins[pluginName] = plugin;
							Object.keys(this.lifecycle).forEach(key => {
								const pluginLife = plugin[key];
								if (pluginLife) {
									this.lifecycle[key].on(pluginLife);
								}
							});
						}
					}
					removePlugin(pluginName) {
						assert(pluginName, "A name is required.");
						const plugin = this.registerPlugins[pluginName];
						assert(plugin, `The plugin "${pluginName}" is not registered.`);
						Object.keys(plugin).forEach(key => {
							if (key !== "name") {
								this.lifecycle[key].remove(plugin[key]);
							}
						});
					}
					// eslint-disable-next-line @typescript-eslint/no-shadow
					inherit({ lifecycle, registerPlugins }) {
						Object.keys(lifecycle).forEach(hookName => {
							assert(
								!this.lifecycle[hookName],
								`The hook "${hookName}" has a conflict and cannot be inherited.`
							);
							this.lifecycle[hookName] = lifecycle[hookName];
						});
						Object.keys(registerPlugins).forEach(pluginName => {
							assert(
								!this.registerPlugins[pluginName],
								`The plugin "${pluginName}" has a conflict and cannot be inherited.`
							);
							this.applyPlugin(registerPlugins[pluginName]);
						});
					}
					constructor(lifecycle) {
						this.registerPlugins = {};
						this.lifecycle = lifecycle;
						this.lifecycleKeys = Object.keys(lifecycle);
					}
				}

				function defaultPreloadArgs(preloadConfig) {
					return polyfills._extends(
						{
							resourceCategory: "sync",
							share: true,
							depsRemote: true,
							prefetchInterface: false
						},
						preloadConfig
					);
				}
				function formatPreloadArgs(remotes, preloadArgs) {
					return preloadArgs.map(args => {
						const remoteInfo = matchRemote(remotes, args.nameOrAlias);
						assert(
							remoteInfo,
							`Unable to preload ${args.nameOrAlias} as it is not included in ${
								!remoteInfo &&
								sdk.safeToString({
									remoteInfo,
									remotes
								})
							}`
						);
						return {
							remote: remoteInfo,
							preloadConfig: defaultPreloadArgs(args)
						};
					});
				}
				function normalizePreloadExposes(exposes) {
					if (!exposes) {
						return [];
					}
					return exposes.map(expose => {
						if (expose === ".") {
							return expose;
						}
						if (expose.startsWith("./")) {
							return expose.replace("./", "");
						}
						return expose;
					});
				}
				function preloadAssets(
					remoteInfo,
					host,
					assets, // It is used to distinguish preload from load remote parallel loading
					useLinkPreload = true
				) {
					const { cssAssets, jsAssetsWithoutEntry, entryAssets } = assets;
					if (host.options.inBrowser) {
						entryAssets.forEach(asset => {
							const { moduleInfo } = asset;
							const module = host.moduleCache.get(remoteInfo.name);
							if (module) {
								getRemoteEntry({
									origin: host,
									remoteInfo: moduleInfo,
									remoteEntryExports: module.remoteEntryExports
								});
							} else {
								getRemoteEntry({
									origin: host,
									remoteInfo: moduleInfo,
									remoteEntryExports: undefined
								});
							}
						});
						if (useLinkPreload) {
							const defaultAttrs = {
								rel: "preload",
								as: "style"
							};
							cssAssets.forEach(cssUrl => {
								const { link: cssEl, needAttach } = sdk.createLink({
									url: cssUrl,
									cb: () => {
										// noop
									},
									attrs: defaultAttrs,
									createLinkHook: (url, attrs) => {
										const res = host.loaderHook.lifecycle.createLink.emit({
											url,
											attrs
										});
										if (res instanceof HTMLLinkElement) {
											return res;
										}
										return;
									}
								});
								needAttach && document.head.appendChild(cssEl);
							});
						} else {
							const defaultAttrs = {
								rel: "stylesheet",
								type: "text/css"
							};
							cssAssets.forEach(cssUrl => {
								const { link: cssEl, needAttach } = sdk.createLink({
									url: cssUrl,
									cb: () => {
										// noop
									},
									attrs: defaultAttrs,
									createLinkHook: (url, attrs) => {
										const res = host.loaderHook.lifecycle.createLink.emit({
											url,
											attrs
										});
										if (res instanceof HTMLLinkElement) {
											return res;
										}
										return;
									},
									needDeleteLink: false
								});
								needAttach && document.head.appendChild(cssEl);
							});
						}
						if (useLinkPreload) {
							const defaultAttrs = {
								rel: "preload",
								as: "script"
							};
							jsAssetsWithoutEntry.forEach(jsUrl => {
								const { link: linkEl, needAttach } = sdk.createLink({
									url: jsUrl,
									cb: () => {
										// noop
									},
									attrs: defaultAttrs,
									createLinkHook: (url, attrs) => {
										const res = host.loaderHook.lifecycle.createLink.emit({
											url,
											attrs
										});
										if (res instanceof HTMLLinkElement) {
											return res;
										}
										return;
									}
								});
								needAttach && document.head.appendChild(linkEl);
							});
						} else {
							const defaultAttrs = {
								fetchpriority: "high",
								type:
									(remoteInfo == null ? void 0 : remoteInfo.type) === "module"
										? "module"
										: "text/javascript"
							};
							jsAssetsWithoutEntry.forEach(jsUrl => {
								const { script: scriptEl, needAttach } = sdk.createScript({
									url: jsUrl,
									cb: () => {
										// noop
									},
									attrs: defaultAttrs,
									createScriptHook: (url, attrs) => {
										const res = host.loaderHook.lifecycle.createScript.emit({
											url,
											attrs
										});
										if (res instanceof HTMLScriptElement) {
											return res;
										}
										return;
									},
									needDeleteScript: true
								});
								needAttach && document.head.appendChild(scriptEl);
							});
						}
					}
				}

				function assignRemoteInfo(remoteInfo, remoteSnapshot) {
					const remoteEntryInfo =
						getRemoteEntryInfoFromSnapshot(remoteSnapshot);
					if (!remoteEntryInfo.url) {
						error(
							`The attribute remoteEntry of ${remoteInfo.name} must not be undefined.`
						);
					}
					let entryUrl = sdk.getResourceUrl(
						remoteSnapshot,
						remoteEntryInfo.url
					);
					if (!sdk.isBrowserEnv() && !entryUrl.startsWith("http")) {
						entryUrl = `https:${entryUrl}`;
					}
					remoteInfo.type = remoteEntryInfo.type;
					remoteInfo.entryGlobalName = remoteEntryInfo.globalName;
					remoteInfo.entry = entryUrl;
					remoteInfo.version = remoteSnapshot.version;
					remoteInfo.buildVersion = remoteSnapshot.buildVersion;
				}
				function snapshotPlugin() {
					return {
						name: "snapshot-plugin",
						async afterResolve(args) {
							const { remote, pkgNameOrAlias, expose, origin, remoteInfo, id } =
								args;
							if (
								!isRemoteInfoWithEntry(remote) ||
								!isPureRemoteEntry(remote)
							) {
								const { remoteSnapshot, globalSnapshot } =
									await origin.snapshotHandler.loadRemoteSnapshotInfo({
										moduleInfo: remote,
										id
									});
								assignRemoteInfo(remoteInfo, remoteSnapshot);
								// preloading assets
								const preloadOptions = {
									remote,
									preloadConfig: {
										nameOrAlias: pkgNameOrAlias,
										exposes: [expose],
										resourceCategory: "sync",
										share: false,
										depsRemote: false
									}
								};
								const assets =
									await origin.remoteHandler.hooks.lifecycle.generatePreloadAssets.emit(
										{
											origin,
											preloadOptions,
											remoteInfo,
											remote,
											remoteSnapshot,
											globalSnapshot
										}
									);
								if (assets) {
									preloadAssets(remoteInfo, origin, assets, false);
								}
								return polyfills._extends({}, args, {
									remoteSnapshot
								});
							}
							return args;
						}
					};
				}

				// name
				// name:version
				function splitId(id) {
					const splitInfo = id.split(":");
					if (splitInfo.length === 1) {
						return {
							name: splitInfo[0],
							version: undefined
						};
					} else if (splitInfo.length === 2) {
						return {
							name: splitInfo[0],
							version: splitInfo[1]
						};
					} else {
						return {
							name: splitInfo[1],
							version: splitInfo[2]
						};
					}
				}
				// Traverse all nodes in moduleInfo and traverse the entire snapshot
				function traverseModuleInfo(
					globalSnapshot,
					remoteInfo,
					traverse,
					isRoot,
					memo = {},
					remoteSnapshot
				) {
					const id = getFMId(remoteInfo);
					const { value: snapshotValue } = getInfoWithoutType(
						globalSnapshot,
						id
					);
					const effectiveRemoteSnapshot = remoteSnapshot || snapshotValue;
					if (
						effectiveRemoteSnapshot &&
						!sdk.isManifestProvider(effectiveRemoteSnapshot)
					) {
						traverse(effectiveRemoteSnapshot, remoteInfo, isRoot);
						if (effectiveRemoteSnapshot.remotesInfo) {
							const remoteKeys = Object.keys(
								effectiveRemoteSnapshot.remotesInfo
							);
							for (const key of remoteKeys) {
								if (memo[key]) {
									continue;
								}
								memo[key] = true;
								const subRemoteInfo = splitId(key);
								const remoteValue = effectiveRemoteSnapshot.remotesInfo[key];
								traverseModuleInfo(
									globalSnapshot,
									{
										name: subRemoteInfo.name,
										version: remoteValue.matchedVersion
									},
									traverse,
									false,
									memo,
									undefined
								);
							}
						}
					}
				}
				const isExisted = (type, url) => {
					return document.querySelector(
						`${type}[${type === "link" ? "href" : "src"}="${url}"]`
					);
				};
				// eslint-disable-next-line max-lines-per-function
				function generatePreloadAssets(
					origin,
					preloadOptions,
					remote,
					globalSnapshot,
					remoteSnapshot
				) {
					const cssAssets = [];
					const jsAssets = [];
					const entryAssets = [];
					const loadedSharedJsAssets = new Set();
					const loadedSharedCssAssets = new Set();
					const { options } = origin;
					const { preloadConfig: rootPreloadConfig } = preloadOptions;
					const { depsRemote } = rootPreloadConfig;
					const memo = {};
					traverseModuleInfo(
						globalSnapshot,
						remote,
						(moduleInfoSnapshot, remoteInfo, isRoot) => {
							let preloadConfig;
							if (isRoot) {
								preloadConfig = rootPreloadConfig;
							} else {
								if (Array.isArray(depsRemote)) {
									// eslint-disable-next-line array-callback-return
									const findPreloadConfig = depsRemote.find(remoteConfig => {
										if (
											remoteConfig.nameOrAlias === remoteInfo.name ||
											remoteConfig.nameOrAlias === remoteInfo.alias
										) {
											return true;
										}
										return false;
									});
									if (!findPreloadConfig) {
										return;
									}
									preloadConfig = defaultPreloadArgs(findPreloadConfig);
								} else if (depsRemote === true) {
									preloadConfig = rootPreloadConfig;
								} else {
									return;
								}
							}
							const remoteEntryUrl = sdk.getResourceUrl(
								moduleInfoSnapshot,
								getRemoteEntryInfoFromSnapshot(moduleInfoSnapshot).url
							);
							if (remoteEntryUrl) {
								entryAssets.push({
									name: remoteInfo.name,
									moduleInfo: {
										name: remoteInfo.name,
										entry: remoteEntryUrl,
										type:
											"remoteEntryType" in moduleInfoSnapshot
												? moduleInfoSnapshot.remoteEntryType
												: "global",
										entryGlobalName:
											"globalName" in moduleInfoSnapshot
												? moduleInfoSnapshot.globalName
												: remoteInfo.name,
										shareScope: "",
										version:
											"version" in moduleInfoSnapshot
												? moduleInfoSnapshot.version
												: undefined
									},
									url: remoteEntryUrl
								});
							}
							let moduleAssetsInfo =
								"modules" in moduleInfoSnapshot
									? moduleInfoSnapshot.modules
									: [];
							const normalizedPreloadExposes = normalizePreloadExposes(
								preloadConfig.exposes
							);
							if (
								normalizedPreloadExposes.length &&
								"modules" in moduleInfoSnapshot
							) {
								var _moduleInfoSnapshot_modules;
								moduleAssetsInfo =
									moduleInfoSnapshot == null
										? void 0
										: (_moduleInfoSnapshot_modules =
													moduleInfoSnapshot.modules) == null
											? void 0
											: _moduleInfoSnapshot_modules.reduce(
													(assets, moduleAssetInfo) => {
														if (
															(normalizedPreloadExposes == null
																? void 0
																: normalizedPreloadExposes.indexOf(
																		moduleAssetInfo.moduleName
																	)) !== -1
														) {
															assets.push(moduleAssetInfo);
														}
														return assets;
													},
													[]
												);
							}
							function handleAssets(assets) {
								const assetsRes = assets.map(asset =>
									sdk.getResourceUrl(moduleInfoSnapshot, asset)
								);
								if (preloadConfig.filter) {
									return assetsRes.filter(preloadConfig.filter);
								}
								return assetsRes;
							}
							if (moduleAssetsInfo) {
								const assetsLength = moduleAssetsInfo.length;
								for (let index = 0; index < assetsLength; index++) {
									const assetsInfo = moduleAssetsInfo[index];
									const exposeFullPath = `${remoteInfo.name}/${assetsInfo.moduleName}`;
									origin.remoteHandler.hooks.lifecycle.handlePreloadModule.emit(
										{
											id:
												assetsInfo.moduleName === "."
													? remoteInfo.name
													: exposeFullPath,
											name: remoteInfo.name,
											remoteSnapshot: moduleInfoSnapshot,
											preloadConfig,
											remote: remoteInfo,
											origin
										}
									);
									const preloaded = getPreloaded(exposeFullPath);
									if (preloaded) {
										continue;
									}
									if (preloadConfig.resourceCategory === "all") {
										cssAssets.push(
											...handleAssets(assetsInfo.assets.css.async)
										);
										cssAssets.push(...handleAssets(assetsInfo.assets.css.sync));
										jsAssets.push(...handleAssets(assetsInfo.assets.js.async));
										jsAssets.push(...handleAssets(assetsInfo.assets.js.sync));
										// eslint-disable-next-line no-constant-condition
									} else if ((preloadConfig.resourceCategory = "sync")) {
										cssAssets.push(...handleAssets(assetsInfo.assets.css.sync));
										jsAssets.push(...handleAssets(assetsInfo.assets.js.sync));
									}
									setPreloaded(exposeFullPath);
								}
							}
						},
						true,
						memo,
						remoteSnapshot
					);
					if (remoteSnapshot.shared) {
						const collectSharedAssets = (shareInfo, snapshotShared) => {
							const registeredShared = getRegisteredShare(
								origin.shareScopeMap,
								snapshotShared.sharedName,
								shareInfo,
								origin.sharedHandler.hooks.lifecycle.resolveShare
							);
							// If the global share does not exist, or the lib function does not exist, it means that the shared has not been loaded yet and can be preloaded.
							if (
								registeredShared &&
								typeof registeredShared.lib === "function"
							) {
								snapshotShared.assets.js.sync.forEach(asset => {
									loadedSharedJsAssets.add(asset);
								});
								snapshotShared.assets.css.sync.forEach(asset => {
									loadedSharedCssAssets.add(asset);
								});
							}
						};
						remoteSnapshot.shared.forEach(shared => {
							var _options_shared;
							const shareInfos =
								(_options_shared = options.shared) == null
									? void 0
									: _options_shared[shared.sharedName];
							if (!shareInfos) {
								return;
							}
							// if no version, preload all shared
							const sharedOptions = shared.version
								? shareInfos.find(s => s.version === shared.version)
								: shareInfos;
							if (!sharedOptions) {
								return;
							}
							const arrayShareInfo = arrayOptions(sharedOptions);
							arrayShareInfo.forEach(s => {
								collectSharedAssets(s, shared);
							});
						});
					}
					const needPreloadJsAssets = jsAssets.filter(
						asset =>
							!loadedSharedJsAssets.has(asset) && !isExisted("script", asset)
					);
					const needPreloadCssAssets = cssAssets.filter(
						asset =>
							!loadedSharedCssAssets.has(asset) && !isExisted("link", asset)
					);
					return {
						cssAssets: needPreloadCssAssets,
						jsAssetsWithoutEntry: needPreloadJsAssets,
						entryAssets: entryAssets.filter(
							entry => !isExisted("script", entry.url)
						)
					};
				}
				const generatePreloadAssetsPlugin = function () {
					return {
						name: "generate-preload-assets-plugin",
						async generatePreloadAssets(args) {
							const {
								origin,
								preloadOptions,
								remoteInfo,
								remote,
								globalSnapshot,
								remoteSnapshot
							} = args;
							if (!sdk.isBrowserEnv()) {
								return {
									cssAssets: [],
									jsAssetsWithoutEntry: [],
									entryAssets: []
								};
							}
							if (isRemoteInfoWithEntry(remote) && isPureRemoteEntry(remote)) {
								return {
									cssAssets: [],
									jsAssetsWithoutEntry: [],
									entryAssets: [
										{
											name: remote.name,
											url: remote.entry,
											moduleInfo: {
												name: remoteInfo.name,
												entry: remote.entry,
												type: remoteInfo.type || "global",
												entryGlobalName: "",
												shareScope: ""
											}
										}
									]
								};
							}
							assignRemoteInfo(remoteInfo, remoteSnapshot);
							const assets = generatePreloadAssets(
								origin,
								preloadOptions,
								remoteInfo,
								globalSnapshot,
								remoteSnapshot
							);
							return assets;
						}
					};
				};

				function getGlobalRemoteInfo(moduleInfo, origin) {
					const hostGlobalSnapshot = getGlobalSnapshotInfoByModuleInfo({
						name: origin.name,
						version: origin.options.version
					});
					// get remote detail info from global
					const globalRemoteInfo =
						hostGlobalSnapshot &&
						"remotesInfo" in hostGlobalSnapshot &&
						hostGlobalSnapshot.remotesInfo &&
						getInfoWithoutType(hostGlobalSnapshot.remotesInfo, moduleInfo.name)
							.value;
					if (globalRemoteInfo && globalRemoteInfo.matchedVersion) {
						return {
							hostGlobalSnapshot,
							globalSnapshot: getGlobalSnapshot(),
							remoteSnapshot: getGlobalSnapshotInfoByModuleInfo({
								name: moduleInfo.name,
								version: globalRemoteInfo.matchedVersion
							})
						};
					}
					return {
						hostGlobalSnapshot: undefined,
						globalSnapshot: getGlobalSnapshot(),
						remoteSnapshot: getGlobalSnapshotInfoByModuleInfo({
							name: moduleInfo.name,
							version: "version" in moduleInfo ? moduleInfo.version : undefined
						})
					};
				}
				class SnapshotHandler {
					// eslint-disable-next-line max-lines-per-function
					async loadRemoteSnapshotInfo({ moduleInfo, id, expose }) {
						const { options } = this.HostInstance;
						await this.hooks.lifecycle.beforeLoadRemoteSnapshot.emit({
							options,
							moduleInfo
						});
						let hostSnapshot = getGlobalSnapshotInfoByModuleInfo({
							name: this.HostInstance.options.name,
							version: this.HostInstance.options.version
						});
						if (!hostSnapshot) {
							hostSnapshot = {
								version: this.HostInstance.options.version || "",
								remoteEntry: "",
								remotesInfo: {}
							};
							addGlobalSnapshot({
								[this.HostInstance.options.name]: hostSnapshot
							});
						}
						// In dynamic loadRemote scenarios, incomplete remotesInfo delivery may occur. In such cases, the remotesInfo in the host needs to be completed in the snapshot at runtime.
						// This ensures the snapshot's integrity and helps the chrome plugin correctly identify all producer modules, ensuring that proxyable producer modules will not be missing.
						if (
							hostSnapshot &&
							"remotesInfo" in hostSnapshot &&
							!getInfoWithoutType(hostSnapshot.remotesInfo, moduleInfo.name)
								.value
						) {
							if ("version" in moduleInfo || "entry" in moduleInfo) {
								hostSnapshot.remotesInfo = polyfills._extends(
									{},
									hostSnapshot == null ? void 0 : hostSnapshot.remotesInfo,
									{
										[moduleInfo.name]: {
											matchedVersion:
												"version" in moduleInfo
													? moduleInfo.version
													: moduleInfo.entry
										}
									}
								);
							}
						}
						const { hostGlobalSnapshot, remoteSnapshot, globalSnapshot } =
							this.getGlobalRemoteInfo(moduleInfo);
						const {
							remoteSnapshot: globalRemoteSnapshot,
							globalSnapshot: globalSnapshotRes
						} = await this.hooks.lifecycle.loadSnapshot.emit({
							options,
							moduleInfo,
							hostGlobalSnapshot,
							remoteSnapshot,
							globalSnapshot
						});
						let mSnapshot;
						let gSnapshot;
						// global snapshot includes manifest or module info includes manifest
						if (globalRemoteSnapshot) {
							if (sdk.isManifestProvider(globalRemoteSnapshot)) {
								const remoteEntry = sdk.isBrowserEnv()
									? globalRemoteSnapshot.remoteEntry
									: globalRemoteSnapshot.ssrRemoteEntry ||
										globalRemoteSnapshot.remoteEntry ||
										"";
								const moduleSnapshot = await this.getManifestJson(
									remoteEntry,
									moduleInfo,
									{}
								);
								// eslint-disable-next-line @typescript-eslint/no-shadow
								const globalSnapshotRes = setGlobalSnapshotInfoByModuleInfo(
									polyfills._extends({}, moduleInfo, {
										// The global remote may be overridden
										// Therefore, set the snapshot key to the global address of the actual request
										entry: remoteEntry
									}),
									moduleSnapshot
								);
								mSnapshot = moduleSnapshot;
								gSnapshot = globalSnapshotRes;
							} else {
								const { remoteSnapshot: remoteSnapshotRes } =
									await this.hooks.lifecycle.loadRemoteSnapshot.emit({
										options: this.HostInstance.options,
										moduleInfo,
										remoteSnapshot: globalRemoteSnapshot,
										from: "global"
									});
								mSnapshot = remoteSnapshotRes;
								gSnapshot = globalSnapshotRes;
							}
						} else {
							if (isRemoteInfoWithEntry(moduleInfo)) {
								// get from manifest.json and merge remote info from remote server
								const moduleSnapshot = await this.getManifestJson(
									moduleInfo.entry,
									moduleInfo,
									{}
								);
								// eslint-disable-next-line @typescript-eslint/no-shadow
								const globalSnapshotRes = setGlobalSnapshotInfoByModuleInfo(
									moduleInfo,
									moduleSnapshot
								);
								const { remoteSnapshot: remoteSnapshotRes } =
									await this.hooks.lifecycle.loadRemoteSnapshot.emit({
										options: this.HostInstance.options,
										moduleInfo,
										remoteSnapshot: moduleSnapshot,
										from: "global"
									});
								mSnapshot = remoteSnapshotRes;
								gSnapshot = globalSnapshotRes;
							} else {
								error(
									errorCodes.getShortErrorMsg(
										errorCodes.RUNTIME_007,
										errorCodes.runtimeDescMap,
										{
											hostName: moduleInfo.name,
											hostVersion: moduleInfo.version,
											globalSnapshot: JSON.stringify(globalSnapshotRes)
										}
									)
								);
							}
						}
						await this.hooks.lifecycle.afterLoadSnapshot.emit({
							id,
							host: this.HostInstance,
							options,
							moduleInfo,
							remoteSnapshot: mSnapshot
						});
						return {
							remoteSnapshot: mSnapshot,
							globalSnapshot: gSnapshot
						};
					}
					getGlobalRemoteInfo(moduleInfo) {
						return getGlobalRemoteInfo(moduleInfo, this.HostInstance);
					}
					async getManifestJson(manifestUrl, moduleInfo, extraOptions) {
						const getManifest = async () => {
							let manifestJson = this.manifestCache.get(manifestUrl);
							if (manifestJson) {
								return manifestJson;
							}
							try {
								let res = await this.loaderHook.lifecycle.fetch.emit(
									manifestUrl,
									{}
								);
								if (!res || !(res instanceof Response)) {
									res = await fetch(manifestUrl, {});
								}
								manifestJson = await res.json();
							} catch (err) {
								manifestJson =
									await this.HostInstance.remoteHandler.hooks.lifecycle.errorLoadRemote.emit(
										{
											id: manifestUrl,
											error: err,
											from: "runtime",
											lifecycle: "afterResolve",
											origin: this.HostInstance
										}
									);
								if (!manifestJson) {
									delete this.manifestLoading[manifestUrl];
									error(
										errorCodes.getShortErrorMsg(
											errorCodes.RUNTIME_003,
											errorCodes.runtimeDescMap,
											{
												manifestUrl,
												moduleName: moduleInfo.name,
												hostName: this.HostInstance.options.name
											},
											`${err}`
										)
									);
								}
							}
							assert(
								manifestJson.metaData &&
									manifestJson.exposes &&
									manifestJson.shared,
								`${manifestUrl} is not a federation manifest`
							);
							this.manifestCache.set(manifestUrl, manifestJson);
							return manifestJson;
						};
						const asyncLoadProcess = async () => {
							const manifestJson = await getManifest();
							const remoteSnapshot = sdk.generateSnapshotFromManifest(
								manifestJson,
								{
									version: manifestUrl
								}
							);
							const { remoteSnapshot: remoteSnapshotRes } =
								await this.hooks.lifecycle.loadRemoteSnapshot.emit({
									options: this.HostInstance.options,
									moduleInfo,
									manifestJson,
									remoteSnapshot,
									manifestUrl,
									from: "manifest"
								});
							return remoteSnapshotRes;
						};
						if (!this.manifestLoading[manifestUrl]) {
							this.manifestLoading[manifestUrl] = asyncLoadProcess().then(
								res => res
							);
						}
						return this.manifestLoading[manifestUrl];
					}
					constructor(HostInstance) {
						this.loadingHostSnapshot = null;
						this.manifestCache = new Map();
						this.hooks = new PluginSystem({
							beforeLoadRemoteSnapshot: new AsyncHook(
								"beforeLoadRemoteSnapshot"
							),
							loadSnapshot: new AsyncWaterfallHook("loadGlobalSnapshot"),
							loadRemoteSnapshot: new AsyncWaterfallHook("loadRemoteSnapshot"),
							afterLoadSnapshot: new AsyncWaterfallHook("afterLoadSnapshot")
						});
						this.manifestLoading = Global.__FEDERATION__.__MANIFEST_LOADING__;
						this.HostInstance = HostInstance;
						this.loaderHook = HostInstance.loaderHook;
					}
				}

				class SharedHandler {
					// register shared in shareScopeMap
					registerShared(globalOptions, userOptions) {
						const { shareInfos, shared } = formatShareConfigs(
							globalOptions,
							userOptions
						);
						const sharedKeys = Object.keys(shareInfos);
						sharedKeys.forEach(sharedKey => {
							const sharedVals = shareInfos[sharedKey];
							sharedVals.forEach(sharedVal => {
								const registeredShared = getRegisteredShare(
									this.shareScopeMap,
									sharedKey,
									sharedVal,
									this.hooks.lifecycle.resolveShare
								);
								if (!registeredShared && sharedVal && sharedVal.lib) {
									this.setShared({
										pkgName: sharedKey,
										lib: sharedVal.lib,
										get: sharedVal.get,
										loaded: true,
										shared: sharedVal,
										from: userOptions.name
									});
								}
							});
						});
						return {
							shareInfos,
							shared
						};
					}
					async loadShare(pkgName, extraOptions) {
						const { host } = this;
						// This function performs the following steps:
						// 1. Checks if the currently loaded share already exists, if not, it throws an error
						// 2. Searches globally for a matching share, if found, it uses it directly
						// 3. If not found, it retrieves it from the current share and stores the obtained share globally.
						const shareInfo = getTargetSharedOptions({
							pkgName,
							extraOptions,
							shareInfos: host.options.shared
						});
						if (shareInfo == null ? void 0 : shareInfo.scope) {
							await Promise.all(
								shareInfo.scope.map(async shareScope => {
									await Promise.all(
										this.initializeSharing(shareScope, {
											strategy: shareInfo.strategy
										})
									);
									return;
								})
							);
						}
						const loadShareRes =
							await this.hooks.lifecycle.beforeLoadShare.emit({
								pkgName,
								shareInfo,
								shared: host.options.shared,
								origin: host
							});
						const { shareInfo: shareInfoRes } = loadShareRes;
						// Assert that shareInfoRes exists, if not, throw an error
						assert(
							shareInfoRes,
							`Cannot find ${pkgName} Share in the ${host.options.name}. Please ensure that the ${pkgName} Share parameters have been injected`
						);
						// Retrieve from cache
						const registeredShared = getRegisteredShare(
							this.shareScopeMap,
							pkgName,
							shareInfoRes,
							this.hooks.lifecycle.resolveShare
						);
						const addUseIn = shared => {
							if (!shared.useIn) {
								shared.useIn = [];
							}
							addUniqueItem(shared.useIn, host.options.name);
						};
						if (registeredShared && registeredShared.lib) {
							addUseIn(registeredShared);
							return registeredShared.lib;
						} else if (
							registeredShared &&
							registeredShared.loading &&
							!registeredShared.loaded
						) {
							const factory = await registeredShared.loading;
							registeredShared.loaded = true;
							if (!registeredShared.lib) {
								registeredShared.lib = factory;
							}
							addUseIn(registeredShared);
							return factory;
						} else if (registeredShared) {
							const asyncLoadProcess = async () => {
								const factory = await registeredShared.get();
								shareInfoRes.lib = factory;
								shareInfoRes.loaded = true;
								addUseIn(shareInfoRes);
								const gShared = getRegisteredShare(
									this.shareScopeMap,
									pkgName,
									shareInfoRes,
									this.hooks.lifecycle.resolveShare
								);
								if (gShared) {
									gShared.lib = factory;
									gShared.loaded = true;
								}
								return factory;
							};
							const loading = asyncLoadProcess();
							this.setShared({
								pkgName,
								loaded: false,
								shared: registeredShared,
								from: host.options.name,
								lib: null,
								loading
							});
							return loading;
						} else {
							if (
								extraOptions == null ? void 0 : extraOptions.customShareInfo
							) {
								return false;
							}
							const asyncLoadProcess = async () => {
								const factory = await shareInfoRes.get();
								shareInfoRes.lib = factory;
								shareInfoRes.loaded = true;
								addUseIn(shareInfoRes);
								const gShared = getRegisteredShare(
									this.shareScopeMap,
									pkgName,
									shareInfoRes,
									this.hooks.lifecycle.resolveShare
								);
								if (gShared) {
									gShared.lib = factory;
									gShared.loaded = true;
								}
								return factory;
							};
							const loading = asyncLoadProcess();
							this.setShared({
								pkgName,
								loaded: false,
								shared: shareInfoRes,
								from: host.options.name,
								lib: null,
								loading
							});
							return loading;
						}
					}
					/**
					 * This function initializes the sharing sequence (executed only once per share scope).
					 * It accepts one argument, the name of the share scope.
					 * If the share scope does not exist, it creates one.
					 */ // eslint-disable-next-line @typescript-eslint/member-ordering
					initializeSharing(shareScopeName = DEFAULT_SCOPE, extraOptions) {
						const { host } = this;
						const from = extraOptions == null ? void 0 : extraOptions.from;
						const strategy =
							extraOptions == null ? void 0 : extraOptions.strategy;
						let initScope =
							extraOptions == null ? void 0 : extraOptions.initScope;
						const promises = [];
						if (from !== "build") {
							const { initTokens } = this;
							if (!initScope) initScope = [];
							let initToken = initTokens[shareScopeName];
							if (!initToken)
								initToken = initTokens[shareScopeName] = {
									from: this.host.name
								};
							if (initScope.indexOf(initToken) >= 0) return promises;
							initScope.push(initToken);
						}
						const shareScope = this.shareScopeMap;
						const hostName = host.options.name;
						// Creates a new share scope if necessary
						if (!shareScope[shareScopeName]) {
							shareScope[shareScopeName] = {};
						}
						// Executes all initialization snippets from all accessible modules
						const scope = shareScope[shareScopeName];
						const register = (name, shared) => {
							var _activeVersion_shareConfig;
							const { version, eager } = shared;
							scope[name] = scope[name] || {};
							const versions = scope[name];
							const activeVersion = versions[version];
							const activeVersionEager = Boolean(
								activeVersion &&
									(activeVersion.eager ||
										((_activeVersion_shareConfig = activeVersion.shareConfig) ==
										null
											? void 0
											: _activeVersion_shareConfig.eager))
							);
							if (
								!activeVersion ||
								(activeVersion.strategy !== "loaded-first" &&
									!activeVersion.loaded &&
									(Boolean(!eager) !== !activeVersionEager
										? eager
										: hostName > activeVersion.from))
							) {
								versions[version] = shared;
							}
						};
						const initFn = mod =>
							mod &&
							mod.init &&
							mod.init(shareScope[shareScopeName], initScope);
						const initRemoteModule = async key => {
							const { module } =
								await host.remoteHandler.getRemoteModuleAndOptions({
									id: key
								});
							if (module.getEntry) {
								let remoteEntryExports;
								try {
									remoteEntryExports = await module.getEntry();
								} catch (error) {
									remoteEntryExports =
										await host.remoteHandler.hooks.lifecycle.errorLoadRemote.emit(
											{
												id: key,
												error,
												from: "runtime",
												lifecycle: "beforeLoadShare",
												origin: host
											}
										);
								}
								if (!module.inited) {
									await initFn(remoteEntryExports);
									module.inited = true;
								}
							}
						};
						Object.keys(host.options.shared).forEach(shareName => {
							const sharedArr = host.options.shared[shareName];
							sharedArr.forEach(shared => {
								if (shared.scope.includes(shareScopeName)) {
									register(shareName, shared);
								}
							});
						});
						// TODO: strategy==='version-first' need to be removed in the future
						if (
							host.options.shareStrategy === "version-first" ||
							strategy === "version-first"
						) {
							host.options.remotes.forEach(remote => {
								if (remote.shareScope === shareScopeName) {
									promises.push(initRemoteModule(remote.name));
								}
							});
						}
						return promises;
					}
					// The lib function will only be available if the shared set by eager or runtime init is set or the shared is successfully loaded.
					// 1. If the loaded shared already exists globally, then it will be reused
					// 2. If lib exists in local shared, it will be used directly
					// 3. If the local get returns something other than Promise, then it will be used directly
					loadShareSync(pkgName, extraOptions) {
						const { host } = this;
						const shareInfo = getTargetSharedOptions({
							pkgName,
							extraOptions,
							shareInfos: host.options.shared
						});
						if (shareInfo == null ? void 0 : shareInfo.scope) {
							shareInfo.scope.forEach(shareScope => {
								this.initializeSharing(shareScope, {
									strategy: shareInfo.strategy
								});
							});
						}
						const registeredShared = getRegisteredShare(
							this.shareScopeMap,
							pkgName,
							shareInfo,
							this.hooks.lifecycle.resolveShare
						);
						const addUseIn = shared => {
							if (!shared.useIn) {
								shared.useIn = [];
							}
							addUniqueItem(shared.useIn, host.options.name);
						};
						if (registeredShared) {
							if (typeof registeredShared.lib === "function") {
								addUseIn(registeredShared);
								if (!registeredShared.loaded) {
									registeredShared.loaded = true;
									if (registeredShared.from === host.options.name) {
										shareInfo.loaded = true;
									}
								}
								return registeredShared.lib;
							}
							if (typeof registeredShared.get === "function") {
								const module = registeredShared.get();
								if (!(module instanceof Promise)) {
									addUseIn(registeredShared);
									this.setShared({
										pkgName,
										loaded: true,
										from: host.options.name,
										lib: module,
										shared: registeredShared
									});
									return module;
								}
							}
						}
						if (shareInfo.lib) {
							if (!shareInfo.loaded) {
								shareInfo.loaded = true;
							}
							return shareInfo.lib;
						}
						if (shareInfo.get) {
							const module = shareInfo.get();
							if (module instanceof Promise) {
								const errorCode =
									(extraOptions == null ? void 0 : extraOptions.from) ===
									"build"
										? errorCodes.RUNTIME_005
										: errorCodes.RUNTIME_006;
								throw new Error(
									errorCodes.getShortErrorMsg(
										errorCode,
										errorCodes.runtimeDescMap,
										{
											hostName: host.options.name,
											sharedPkgName: pkgName
										}
									)
								);
							}
							shareInfo.lib = module;
							this.setShared({
								pkgName,
								loaded: true,
								from: host.options.name,
								lib: shareInfo.lib,
								shared: shareInfo
							});
							return shareInfo.lib;
						}
						throw new Error(
							errorCodes.getShortErrorMsg(
								errorCodes.RUNTIME_006,
								errorCodes.runtimeDescMap,
								{
									hostName: host.options.name,
									sharedPkgName: pkgName
								}
							)
						);
					}
					initShareScopeMap(scopeName, shareScope, extraOptions = {}) {
						const { host } = this;
						this.shareScopeMap[scopeName] = shareScope;
						this.hooks.lifecycle.initContainerShareScopeMap.emit({
							shareScope,
							options: host.options,
							origin: host,
							scopeName,
							hostShareScopeMap: extraOptions.hostShareScopeMap
						});
					}
					setShared({ pkgName, shared, from, lib, loading, loaded, get }) {
						const { version, scope = "default" } = shared,
							shareInfo = polyfills._object_without_properties_loose(shared, [
								"version",
								"scope"
							]);
						const scopes = Array.isArray(scope) ? scope : [scope];
						scopes.forEach(sc => {
							if (!this.shareScopeMap[sc]) {
								this.shareScopeMap[sc] = {};
							}
							if (!this.shareScopeMap[sc][pkgName]) {
								this.shareScopeMap[sc][pkgName] = {};
							}
							if (!this.shareScopeMap[sc][pkgName][version]) {
								this.shareScopeMap[sc][pkgName][version] = polyfills._extends(
									{
										version,
										scope: ["default"]
									},
									shareInfo,
									{
										lib,
										loaded,
										loading
									}
								);
								if (get) {
									this.shareScopeMap[sc][pkgName][version].get = get;
								}
								return;
							}
							const registeredShared = this.shareScopeMap[sc][pkgName][version];
							if (loading && !registeredShared.loading) {
								registeredShared.loading = loading;
							}
						});
					}
					_setGlobalShareScopeMap(hostOptions) {
						const globalShareScopeMap = getGlobalShareScope();
						const identifier = hostOptions.id || hostOptions.name;
						if (identifier && !globalShareScopeMap[identifier]) {
							globalShareScopeMap[identifier] = this.shareScopeMap;
						}
					}
					constructor(host) {
						this.hooks = new PluginSystem({
							afterResolve: new AsyncWaterfallHook("afterResolve"),
							beforeLoadShare: new AsyncWaterfallHook("beforeLoadShare"),
							// not used yet
							loadShare: new AsyncHook(),
							resolveShare: new SyncWaterfallHook("resolveShare"),
							// maybe will change, temporarily for internal use only
							initContainerShareScopeMap: new SyncWaterfallHook(
								"initContainerShareScopeMap"
							)
						});
						this.host = host;
						this.shareScopeMap = {};
						this.initTokens = {};
						this._setGlobalShareScopeMap(host.options);
					}
				}

				class RemoteHandler {
					formatAndRegisterRemote(globalOptions, userOptions) {
						const userRemotes = userOptions.remotes || [];
						return userRemotes.reduce((res, remote) => {
							this.registerRemote(remote, res, {
								force: false
							});
							return res;
						}, globalOptions.remotes);
					}
					setIdToRemoteMap(id, remoteMatchInfo) {
						const { remote, expose } = remoteMatchInfo;
						const { name, alias } = remote;
						this.idToRemoteMap[id] = {
							name: remote.name,
							expose
						};
						if (alias && id.startsWith(name)) {
							const idWithAlias = id.replace(name, alias);
							this.idToRemoteMap[idWithAlias] = {
								name: remote.name,
								expose
							};
							return;
						}
						if (alias && id.startsWith(alias)) {
							const idWithName = id.replace(alias, name);
							this.idToRemoteMap[idWithName] = {
								name: remote.name,
								expose
							};
						}
					}
					// eslint-disable-next-line max-lines-per-function
					// eslint-disable-next-line @typescript-eslint/member-ordering
					async loadRemote(id, options) {
						const { host } = this;
						try {
							const { loadFactory = true } = options || {
								loadFactory: true
							};
							// 1. Validate the parameters of the retrieved module. There are two module request methods: pkgName + expose and alias + expose.
							// 2. Request the snapshot information of the current host and globally store the obtained snapshot information. The retrieved module information is partially offline and partially online. The online module information will retrieve the modules used online.
							// 3. Retrieve the detailed information of the current module from global (remoteEntry address, expose resource address)
							// 4. After retrieving remoteEntry, call the init of the module, and then retrieve the exported content of the module through get
							// id: pkgName(@federation/app1) + expose(button) = @federation/app1/button
							// id: alias(app1) + expose(button) = app1/button
							// id: alias(app1/utils) + expose(loadash/sort) = app1/utils/loadash/sort
							const { module, moduleOptions, remoteMatchInfo } =
								await this.getRemoteModuleAndOptions({
									id
								});
							const {
								pkgNameOrAlias,
								remote,
								expose,
								id: idRes,
								remoteSnapshot
							} = remoteMatchInfo;
							const moduleOrFactory = await module.get(
								idRes,
								expose,
								options,
								remoteSnapshot
							);
							const moduleWrapper = await this.hooks.lifecycle.onLoad.emit({
								id: idRes,
								pkgNameOrAlias,
								expose,
								exposeModule: loadFactory ? moduleOrFactory : undefined,
								exposeModuleFactory: loadFactory ? undefined : moduleOrFactory,
								remote,
								options: moduleOptions,
								moduleInstance: module,
								origin: host
							});
							this.setIdToRemoteMap(id, remoteMatchInfo);
							if (typeof moduleWrapper === "function") {
								return moduleWrapper;
							}
							return moduleOrFactory;
						} catch (error) {
							const { from = "runtime" } = options || {
								from: "runtime"
							};
							const failOver = await this.hooks.lifecycle.errorLoadRemote.emit({
								id,
								error,
								from,
								lifecycle: "onLoad",
								origin: host
							});
							if (!failOver) {
								throw error;
							}
							return failOver;
						}
					}
					// eslint-disable-next-line @typescript-eslint/member-ordering
					async preloadRemote(preloadOptions) {
						const { host } = this;
						await this.hooks.lifecycle.beforePreloadRemote.emit({
							preloadOps: preloadOptions,
							options: host.options,
							origin: host
						});
						const preloadOps = formatPreloadArgs(
							host.options.remotes,
							preloadOptions
						);
						await Promise.all(
							preloadOps.map(async ops => {
								const { remote } = ops;
								const remoteInfo = getRemoteInfo(remote);
								const { globalSnapshot, remoteSnapshot } =
									await host.snapshotHandler.loadRemoteSnapshotInfo({
										moduleInfo: remote
									});
								const assets =
									await this.hooks.lifecycle.generatePreloadAssets.emit({
										origin: host,
										preloadOptions: ops,
										remote,
										remoteInfo,
										globalSnapshot,
										remoteSnapshot
									});
								if (!assets) {
									return;
								}
								preloadAssets(remoteInfo, host, assets);
							})
						);
					}
					registerRemotes(remotes, options) {
						const { host } = this;
						remotes.forEach(remote => {
							this.registerRemote(remote, host.options.remotes, {
								force: options == null ? void 0 : options.force
							});
						});
					}
					async getRemoteModuleAndOptions(options) {
						const { host } = this;
						const { id } = options;
						let loadRemoteArgs;
						try {
							loadRemoteArgs = await this.hooks.lifecycle.beforeRequest.emit({
								id,
								options: host.options,
								origin: host
							});
						} catch (error) {
							loadRemoteArgs = await this.hooks.lifecycle.errorLoadRemote.emit({
								id,
								options: host.options,
								origin: host,
								from: "runtime",
								error,
								lifecycle: "beforeRequest"
							});
							if (!loadRemoteArgs) {
								throw error;
							}
						}
						const { id: idRes } = loadRemoteArgs;
						const remoteSplitInfo = matchRemoteWithNameAndExpose(
							host.options.remotes,
							idRes
						);
						assert(
							remoteSplitInfo,
							errorCodes.getShortErrorMsg(
								errorCodes.RUNTIME_004,
								errorCodes.runtimeDescMap,
								{
									hostName: host.options.name,
									requestId: idRes
								}
							)
						);
						const { remote: rawRemote } = remoteSplitInfo;
						const remoteInfo = getRemoteInfo(rawRemote);
						const matchInfo =
							await host.sharedHandler.hooks.lifecycle.afterResolve.emit(
								polyfills._extends(
									{
										id: idRes
									},
									remoteSplitInfo,
									{
										options: host.options,
										origin: host,
										remoteInfo
									}
								)
							);
						const { remote, expose } = matchInfo;
						assert(
							remote && expose,
							`The 'beforeRequest' hook was executed, but it failed to return the correct 'remote' and 'expose' values while loading ${idRes}.`
						);
						let module = host.moduleCache.get(remote.name);
						const moduleOptions = {
							host: host,
							remoteInfo
						};
						if (!module) {
							module = new Module(moduleOptions);
							host.moduleCache.set(remote.name, module);
						}
						return {
							module,
							moduleOptions,
							remoteMatchInfo: matchInfo
						};
					}
					registerRemote(remote, targetRemotes, options) {
						const { host } = this;
						const normalizeRemote = () => {
							if (remote.alias) {
								// Validate if alias equals the prefix of remote.name and remote.alias, if so, throw an error
								// As multi-level path references cannot guarantee unique names, alias being a prefix of remote.name is not supported
								const findEqual = targetRemotes.find(item => {
									var _item_alias;
									return (
										remote.alias &&
										(item.name.startsWith(remote.alias) ||
											((_item_alias = item.alias) == null
												? void 0
												: _item_alias.startsWith(remote.alias)))
									);
								});
								assert(
									!findEqual,
									`The alias ${remote.alias} of remote ${remote.name} is not allowed to be the prefix of ${findEqual && findEqual.name} name or alias`
								);
							}
							// Set the remote entry to a complete path
							if ("entry" in remote) {
								if (sdk.isBrowserEnv() && !remote.entry.startsWith("http")) {
									remote.entry = new URL(
										remote.entry,
										window.location.origin
									).href;
								}
							}
							if (!remote.shareScope) {
								remote.shareScope = DEFAULT_SCOPE;
							}
							if (!remote.type) {
								remote.type = DEFAULT_REMOTE_TYPE;
							}
						};
						this.hooks.lifecycle.beforeRegisterRemote.emit({
							remote,
							origin: host
						});
						const registeredRemote = targetRemotes.find(
							item => item.name === remote.name
						);
						if (!registeredRemote) {
							normalizeRemote();
							targetRemotes.push(remote);
							this.hooks.lifecycle.registerRemote.emit({
								remote,
								origin: host
							});
						} else {
							const messages = [
								`The remote "${remote.name}" is already registered.`,
								"Please note that overriding it may cause unexpected errors."
							];
							if (options == null ? void 0 : options.force) {
								// remove registered remote
								this.removeRemote(registeredRemote);
								normalizeRemote();
								targetRemotes.push(remote);
								this.hooks.lifecycle.registerRemote.emit({
									remote,
									origin: host
								});
								sdk.warn(messages.join(" "));
							}
						}
					}
					removeRemote(remote) {
						try {
							const { host } = this;
							const { name } = remote;
							const remoteIndex = host.options.remotes.findIndex(
								item => item.name === name
							);
							if (remoteIndex !== -1) {
								host.options.remotes.splice(remoteIndex, 1);
							}
							const loadedModule = host.moduleCache.get(remote.name);
							if (loadedModule) {
								const remoteInfo = loadedModule.remoteInfo;
								const key = remoteInfo.entryGlobalName;
								if (CurrentGlobal[key]) {
									var _Object_getOwnPropertyDescriptor;
									if (
										(_Object_getOwnPropertyDescriptor =
											Object.getOwnPropertyDescriptor(CurrentGlobal, key)) ==
										null
											? void 0
											: _Object_getOwnPropertyDescriptor.configurable
									) {
										delete CurrentGlobal[key];
									} else {
										// @ts-ignore
										CurrentGlobal[key] = undefined;
									}
								}
								const remoteEntryUniqueKey = getRemoteEntryUniqueKey(
									loadedModule.remoteInfo
								);
								if (globalLoading[remoteEntryUniqueKey]) {
									delete globalLoading[remoteEntryUniqueKey];
								}
								host.snapshotHandler.manifestCache.delete(remoteInfo.entry);
								// delete unloaded shared and instance
								let remoteInsId = remoteInfo.buildVersion
									? sdk.composeKeyWithSeparator(
											remoteInfo.name,
											remoteInfo.buildVersion
										)
									: remoteInfo.name;
								const remoteInsIndex =
									CurrentGlobal.__FEDERATION__.__INSTANCES__.findIndex(ins => {
										if (remoteInfo.buildVersion) {
											return ins.options.id === remoteInsId;
										} else {
											return ins.name === remoteInsId;
										}
									});
								if (remoteInsIndex !== -1) {
									const remoteIns =
										CurrentGlobal.__FEDERATION__.__INSTANCES__[remoteInsIndex];
									remoteInsId = remoteIns.options.id || remoteInsId;
									const globalShareScopeMap = getGlobalShareScope();
									let isAllSharedNotUsed = true;
									const needDeleteKeys = [];
									Object.keys(globalShareScopeMap).forEach(instId => {
										const shareScopeMap = globalShareScopeMap[instId];
										shareScopeMap &&
											Object.keys(shareScopeMap).forEach(shareScope => {
												const shareScopeVal = shareScopeMap[shareScope];
												shareScopeVal &&
													Object.keys(shareScopeVal).forEach(shareName => {
														const sharedPkgs = shareScopeVal[shareName];
														sharedPkgs &&
															Object.keys(sharedPkgs).forEach(shareVersion => {
																const shared = sharedPkgs[shareVersion];
																if (
																	shared &&
																	typeof shared === "object" &&
																	shared.from === remoteInfo.name
																) {
																	if (shared.loaded || shared.loading) {
																		shared.useIn = shared.useIn.filter(
																			usedHostName =>
																				usedHostName !== remoteInfo.name
																		);
																		if (shared.useIn.length) {
																			isAllSharedNotUsed = false;
																		} else {
																			needDeleteKeys.push([
																				instId,
																				shareScope,
																				shareName,
																				shareVersion
																			]);
																		}
																	} else {
																		needDeleteKeys.push([
																			instId,
																			shareScope,
																			shareName,
																			shareVersion
																		]);
																	}
																}
															});
													});
											});
									});
									if (isAllSharedNotUsed) {
										remoteIns.shareScopeMap = {};
										delete globalShareScopeMap[remoteInsId];
									}
									needDeleteKeys.forEach(
										([insId, shareScope, shareName, shareVersion]) => {
											var _globalShareScopeMap_insId_shareScope_shareName,
												_globalShareScopeMap_insId_shareScope,
												_globalShareScopeMap_insId;
											(_globalShareScopeMap_insId =
												globalShareScopeMap[insId]) == null
												? true
												: (_globalShareScopeMap_insId_shareScope =
															_globalShareScopeMap_insId[shareScope]) == null
													? true
													: (_globalShareScopeMap_insId_shareScope_shareName =
																_globalShareScopeMap_insId_shareScope[
																	shareName
																]) == null
														? true
														: delete _globalShareScopeMap_insId_shareScope_shareName[
																shareVersion
															];
										}
									);
									CurrentGlobal.__FEDERATION__.__INSTANCES__.splice(
										remoteInsIndex,
										1
									);
								}
								const { hostGlobalSnapshot } = getGlobalRemoteInfo(
									remote,
									host
								);
								if (hostGlobalSnapshot) {
									const remoteKey =
										hostGlobalSnapshot &&
										"remotesInfo" in hostGlobalSnapshot &&
										hostGlobalSnapshot.remotesInfo &&
										getInfoWithoutType(
											hostGlobalSnapshot.remotesInfo,
											remote.name
										).key;
									if (remoteKey) {
										delete hostGlobalSnapshot.remotesInfo[remoteKey];
										if (
											//eslint-disable-next-line no-extra-boolean-cast
											Boolean(
												Global.__FEDERATION__.__MANIFEST_LOADING__[remoteKey]
											)
										) {
											delete Global.__FEDERATION__.__MANIFEST_LOADING__[
												remoteKey
											];
										}
									}
								}
								host.moduleCache.delete(remote.name);
							}
						} catch (err) {
							logger.log("removeRemote fail: ", err);
						}
					}
					constructor(host) {
						this.hooks = new PluginSystem({
							beforeRegisterRemote: new SyncWaterfallHook(
								"beforeRegisterRemote"
							),
							registerRemote: new SyncWaterfallHook("registerRemote"),
							beforeRequest: new AsyncWaterfallHook("beforeRequest"),
							onLoad: new AsyncHook("onLoad"),
							handlePreloadModule: new SyncHook("handlePreloadModule"),
							errorLoadRemote: new AsyncHook("errorLoadRemote"),
							beforePreloadRemote: new AsyncHook("beforePreloadRemote"),
							generatePreloadAssets: new AsyncHook("generatePreloadAssets"),
							// not used yet
							afterPreloadRemote: new AsyncHook(),
							loadEntry: new AsyncHook()
						});
						this.host = host;
						this.idToRemoteMap = {};
					}
				}

				const USE_SNAPSHOT =
					typeof FEDERATION_OPTIMIZE_NO_SNAPSHOT_PLUGIN === "boolean"
						? !FEDERATION_OPTIMIZE_NO_SNAPSHOT_PLUGIN
						: true; // Default to true (use snapshot) when not explicitly defined
				class FederationHost {
					initOptions(userOptions) {
						this.registerPlugins(userOptions.plugins);
						const options = this.formatOptions(this.options, userOptions);
						this.options = options;
						return options;
					}
					async loadShare(pkgName, extraOptions) {
						return this.sharedHandler.loadShare(pkgName, extraOptions);
					}
					// The lib function will only be available if the shared set by eager or runtime init is set or the shared is successfully loaded.
					// 1. If the loaded shared already exists globally, then it will be reused
					// 2. If lib exists in local shared, it will be used directly
					// 3. If the local get returns something other than Promise, then it will be used directly
					loadShareSync(pkgName, extraOptions) {
						return this.sharedHandler.loadShareSync(pkgName, extraOptions);
					}
					initializeSharing(shareScopeName = DEFAULT_SCOPE, extraOptions) {
						return this.sharedHandler.initializeSharing(
							shareScopeName,
							extraOptions
						);
					}
					initRawContainer(name, url, container) {
						const remoteInfo = getRemoteInfo({
							name,
							entry: url
						});
						const module = new Module({
							host: this,
							remoteInfo
						});
						module.remoteEntryExports = container;
						this.moduleCache.set(name, module);
						return module;
					}
					// eslint-disable-next-line max-lines-per-function
					// eslint-disable-next-line @typescript-eslint/member-ordering
					async loadRemote(id, options) {
						return this.remoteHandler.loadRemote(id, options);
					}
					// eslint-disable-next-line @typescript-eslint/member-ordering
					async preloadRemote(preloadOptions) {
						return this.remoteHandler.preloadRemote(preloadOptions);
					}
					initShareScopeMap(scopeName, shareScope, extraOptions = {}) {
						this.sharedHandler.initShareScopeMap(
							scopeName,
							shareScope,
							extraOptions
						);
					}
					formatOptions(globalOptions, userOptions) {
						const { shared } = formatShareConfigs(globalOptions, userOptions);
						const { userOptions: userOptionsRes, options: globalOptionsRes } =
							this.hooks.lifecycle.beforeInit.emit({
								origin: this,
								userOptions,
								options: globalOptions,
								shareInfo: shared
							});
						const remotes = this.remoteHandler.formatAndRegisterRemote(
							globalOptionsRes,
							userOptionsRes
						);
						const { shared: handledShared } = this.sharedHandler.registerShared(
							globalOptionsRes,
							userOptionsRes
						);
						const plugins = [...globalOptionsRes.plugins];
						if (userOptionsRes.plugins) {
							userOptionsRes.plugins.forEach(plugin => {
								if (!plugins.includes(plugin)) {
									plugins.push(plugin);
								}
							});
						}
						const optionsRes = polyfills._extends(
							{},
							globalOptions,
							userOptions,
							{
								plugins,
								remotes,
								shared: handledShared
							}
						);
						this.hooks.lifecycle.init.emit({
							origin: this,
							options: optionsRes
						});
						return optionsRes;
					}
					registerPlugins(plugins) {
						const pluginRes = registerPlugins(plugins, [
							this.hooks,
							this.remoteHandler.hooks,
							this.sharedHandler.hooks,
							this.snapshotHandler.hooks,
							this.loaderHook,
							this.bridgeHook
						]);
						// Merge plugin
						this.options.plugins = this.options.plugins.reduce(
							(res, plugin) => {
								if (!plugin) return res;
								if (res && !res.find(item => item.name === plugin.name)) {
									res.push(plugin);
								}
								return res;
							},
							pluginRes || []
						);
					}
					registerRemotes(remotes, options) {
						return this.remoteHandler.registerRemotes(remotes, options);
					}
					constructor(userOptions) {
						this.hooks = new PluginSystem({
							beforeInit: new SyncWaterfallHook("beforeInit"),
							init: new SyncHook(),
							// maybe will change, temporarily for internal use only
							beforeInitContainer: new AsyncWaterfallHook(
								"beforeInitContainer"
							),
							// maybe will change, temporarily for internal use only
							initContainer: new AsyncWaterfallHook("initContainer")
						});
						this.version = "0.16.0";
						this.moduleCache = new Map();
						this.loaderHook = new PluginSystem({
							// FIXME: may not be suitable , not open to the public yet
							getModuleInfo: new SyncHook(),
							createScript: new SyncHook(),
							createLink: new SyncHook(),
							fetch: new AsyncHook(),
							loadEntryError: new AsyncHook(),
							getModuleFactory: new AsyncHook()
						});
						this.bridgeHook = new PluginSystem({
							beforeBridgeRender: new SyncHook(),
							afterBridgeRender: new SyncHook(),
							beforeBridgeDestroy: new SyncHook(),
							afterBridgeDestroy: new SyncHook()
						});
						const plugins = USE_SNAPSHOT
							? [snapshotPlugin(), generatePreloadAssetsPlugin()]
							: [];
						// TODO: Validate the details of the options
						// Initialize options with default values
						const defaultOptions = {
							id: getBuilderId(),
							name: userOptions.name,
							plugins,
							remotes: [],
							shared: {},
							inBrowser: sdk.isBrowserEnv()
						};
						this.name = userOptions.name;
						this.options = defaultOptions;
						this.snapshotHandler = new SnapshotHandler(this);
						this.sharedHandler = new SharedHandler(this);
						this.remoteHandler = new RemoteHandler(this);
						this.shareScopeMap = this.sharedHandler.shareScopeMap;
						this.registerPlugins([
							...defaultOptions.plugins,
							...(userOptions.plugins || [])
						]);
						this.options = this.formatOptions(defaultOptions, userOptions);
					}
				}

				var index = /*#__PURE__*/ Object.freeze({
					__proto__: null
				});

				exports.loadScript = sdk.loadScript;
				exports.loadScriptNode = sdk.loadScriptNode;
				exports.CurrentGlobal = CurrentGlobal;
				exports.FederationHost = FederationHost;
				exports.Global = Global;
				exports.Module = Module;
				exports.addGlobalSnapshot = addGlobalSnapshot;
				exports.assert = assert;
				exports.getGlobalFederationConstructor = getGlobalFederationConstructor;
				exports.getGlobalSnapshot = getGlobalSnapshot;
				exports.getInfoWithoutType = getInfoWithoutType;
				exports.getRegisteredShare = getRegisteredShare;
				exports.getRemoteEntry = getRemoteEntry;
				exports.getRemoteInfo = getRemoteInfo;
				exports.helpers = helpers;
				exports.isStaticResourcesEqual = isStaticResourcesEqual;
				exports.matchRemoteWithNameAndExpose = matchRemoteWithNameAndExpose;
				exports.registerGlobalPlugins = registerGlobalPlugins;
				exports.resetFederationGlobalInfo = resetFederationGlobalInfo;
				exports.safeWrapper = safeWrapper;
				exports.satisfy = satisfy;
				exports.setGlobalFederationConstructor = setGlobalFederationConstructor;
				exports.setGlobalFederationInstance = setGlobalFederationInstance;
				exports.types = index;
			},
		"../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/polyfills.cjs.cjs":
			/*!*******************************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/polyfills.cjs.cjs ***!
  \*******************************************************************************************************************************************/
			function (__unused_webpack_module, exports) {
				function _extends() {
					_extends =
						Object.assign ||
						function assign(target) {
							for (var i = 1; i < arguments.length; i++) {
								var source = arguments[i];
								for (var key in source)
									if (Object.prototype.hasOwnProperty.call(source, key))
										target[key] = source[key];
							}
							return target;
						};
					return _extends.apply(this, arguments);
				}

				function _object_without_properties_loose(source, excluded) {
					if (source == null) return {};
					var target = {};
					var sourceKeys = Object.keys(source);
					var key, i;
					for (i = 0; i < sourceKeys.length; i++) {
						key = sourceKeys[i];
						if (excluded.indexOf(key) >= 0) continue;
						target[key] = source[key];
					}
					return target;
				}

				exports._extends = _extends;
				exports._object_without_properties_loose =
					_object_without_properties_loose;
			},
		"../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/index.cjs.cjs":
			/*!*****************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/index.cjs.cjs ***!
  \*****************************************************************************************************************************/
			function (__unused_webpack_module, exports, __webpack_require__) {
				var runtimeCore = __webpack_require__(
					/*! @module-federation/runtime-core */ "../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/index.cjs.cjs"
				);
				var utils = __webpack_require__(
					/*! ./utils.cjs.cjs */ "../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/utils.cjs.cjs"
				);

				let FederationInstance = null;
				function init(options) {
					// Retrieve the same instance with the same name
					const instance = utils.getGlobalFederationInstance(
						options.name,
						options.version
					);
					if (!instance) {
						// Retrieve debug constructor
						const FederationConstructor =
							runtimeCore.getGlobalFederationConstructor() ||
							runtimeCore.FederationHost;
						FederationInstance = new FederationConstructor(options);
						runtimeCore.setGlobalFederationInstance(FederationInstance);
						return FederationInstance;
					} else {
						// Merge options
						instance.initOptions(options);
						if (!FederationInstance) {
							FederationInstance = instance;
						}
						return instance;
					}
				}
				function loadRemote(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					const loadRemote1 = FederationInstance.loadRemote;
					// eslint-disable-next-line prefer-spread
					return loadRemote1.apply(FederationInstance, args);
				}
				function loadShare(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					// eslint-disable-next-line prefer-spread
					const loadShare1 = FederationInstance.loadShare;
					return loadShare1.apply(FederationInstance, args);
				}
				function loadShareSync(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					const loadShareSync1 = FederationInstance.loadShareSync;
					// eslint-disable-next-line prefer-spread
					return loadShareSync1.apply(FederationInstance, args);
				}
				function preloadRemote(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					// eslint-disable-next-line prefer-spread
					return FederationInstance.preloadRemote.apply(
						FederationInstance,
						args
					);
				}
				function registerRemotes(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					// eslint-disable-next-line prefer-spread
					return FederationInstance.registerRemotes.apply(
						FederationInstance,
						args
					);
				}
				function registerPlugins(...args) {
					runtimeCore.assert(FederationInstance, "Please call init first");
					// eslint-disable-next-line prefer-spread
					return FederationInstance.registerPlugins.apply(
						FederationInstance,
						args
					);
				}
				function getInstance() {
					return FederationInstance;
				}
				// Inject for debug
				runtimeCore.setGlobalFederationConstructor(runtimeCore.FederationHost);

				exports.FederationHost = runtimeCore.FederationHost;
				exports.Module = runtimeCore.Module;
				exports.getRemoteEntry = runtimeCore.getRemoteEntry;
				exports.getRemoteInfo = runtimeCore.getRemoteInfo;
				exports.loadScript = runtimeCore.loadScript;
				exports.loadScriptNode = runtimeCore.loadScriptNode;
				exports.registerGlobalPlugins = runtimeCore.registerGlobalPlugins;
				exports.getInstance = getInstance;
				exports.init = init;
				exports.loadRemote = loadRemote;
				exports.loadShare = loadShare;
				exports.loadShareSync = loadShareSync;
				exports.preloadRemote = preloadRemote;
				exports.registerPlugins = registerPlugins;
				exports.registerRemotes = registerRemotes;
			},
		"../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/utils.cjs.cjs":
			/*!*****************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/utils.cjs.cjs ***!
  \*****************************************************************************************************************************/
			function (__unused_webpack_module, exports, __webpack_require__) {
				var runtimeCore = __webpack_require__(
					/*! @module-federation/runtime-core */ "../../node_modules/.pnpm/@module-federation+runtime-core@0.16.0/node_modules/@module-federation/runtime-core/dist/index.cjs.cjs"
				);

				// injected by bundler, so it can not use runtime-core stuff
				function getBuilderId() {
					//@ts-ignore
					return typeof FEDERATION_BUILD_IDENTIFIER !== "undefined"
						? FEDERATION_BUILD_IDENTIFIER
						: "";
				}
				function getGlobalFederationInstance(name, version) {
					const buildId = getBuilderId();
					return runtimeCore.CurrentGlobal.__FEDERATION__.__INSTANCES__.find(
						GMInstance => {
							if (buildId && GMInstance.options.id === getBuilderId()) {
								return true;
							}
							if (
								GMInstance.options.name === name &&
								!GMInstance.options.version &&
								!version
							) {
								return true;
							}
							if (
								GMInstance.options.name === name &&
								version &&
								GMInstance.options.version === version
							) {
								return true;
							}
							return false;
						}
					);
				}

				exports.getGlobalFederationInstance = getGlobalFederationInstance;
			},
		"../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/index.cjs.cjs":
			/*!*********************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/index.cjs.cjs ***!
  \*********************************************************************************************************************/
			function (__unused_webpack_module, exports, __webpack_require__) {
				var polyfills = __webpack_require__(
					/*! ./polyfills.cjs.cjs */ "../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/polyfills.cjs.cjs"
				);

				const FederationModuleManifest = "federation-manifest.json";
				const MANIFEST_EXT = ".json";
				const BROWSER_LOG_KEY = "FEDERATION_DEBUG";
				const NameTransformSymbol = {
					AT: "@",
					HYPHEN: "-",
					SLASH: "/"
				};
				const NameTransformMap = {
					[NameTransformSymbol.AT]: "scope_",
					[NameTransformSymbol.HYPHEN]: "_",
					[NameTransformSymbol.SLASH]: "__"
				};
				const EncodedNameTransformMap = {
					[NameTransformMap[NameTransformSymbol.AT]]: NameTransformSymbol.AT,
					[NameTransformMap[NameTransformSymbol.HYPHEN]]:
						NameTransformSymbol.HYPHEN,
					[NameTransformMap[NameTransformSymbol.SLASH]]:
						NameTransformSymbol.SLASH
				};
				const SEPARATOR = ":";
				const ManifestFileName = "mf-manifest.json";
				const StatsFileName = "mf-stats.json";
				const MFModuleType = {
					NPM: "npm",
					APP: "app"
				};
				const MODULE_DEVTOOL_IDENTIFIER = "__MF_DEVTOOLS_MODULE_INFO__";
				const ENCODE_NAME_PREFIX = "ENCODE_NAME_PREFIX";
				const TEMP_DIR = ".federation";
				const MFPrefetchCommon = {
					identifier: "MFDataPrefetch",
					globalKey: "__PREFETCH__",
					library: "mf-data-prefetch",
					exportsKey: "__PREFETCH_EXPORTS__",
					fileName: "bootstrap.js"
				};

				var ContainerPlugin = /*#__PURE__*/ Object.freeze({
					__proto__: null
				});

				var ContainerReferencePlugin = /*#__PURE__*/ Object.freeze({
					__proto__: null
				});

				var ModuleFederationPlugin = /*#__PURE__*/ Object.freeze({
					__proto__: null
				});

				var SharePlugin = /*#__PURE__*/ Object.freeze({
					__proto__: null
				});

				function isBrowserEnv() {
					return (
						typeof window !== "undefined" &&
						typeof window.document !== "undefined"
					);
				}
				function isReactNativeEnv() {
					var _navigator;
					return (
						typeof navigator !== "undefined" &&
						((_navigator = navigator) == null ? void 0 : _navigator.product) ===
							"ReactNative"
					);
				}
				function isBrowserDebug() {
					try {
						if (isBrowserEnv() && window.localStorage) {
							return Boolean(localStorage.getItem(BROWSER_LOG_KEY));
						}
					} catch (error) {
						return false;
					}
					return false;
				}
				function isDebugMode() {
					if (
						typeof process !== "undefined" &&
						process.env &&
						process.env["FEDERATION_DEBUG"]
					) {
						return Boolean(process.env["FEDERATION_DEBUG"]);
					}
					if (
						typeof FEDERATION_DEBUG !== "undefined" &&
						Boolean(FEDERATION_DEBUG)
					) {
						return true;
					}
					return isBrowserDebug();
				}
				const getProcessEnv = function () {
					return typeof process !== "undefined" && process.env
						? process.env
						: {};
				};

				const LOG_CATEGORY = "[ Federation Runtime ]";
				// entry: name:version   version : 1.0.0 | ^1.2.3
				// entry: name:entry  entry:  https://localhost:9000/federation-manifest.json
				const parseEntry = (str, devVerOrUrl, separator = SEPARATOR) => {
					const strSplit = str.split(separator);
					const devVersionOrUrl =
						getProcessEnv()["NODE_ENV"] === "development" && devVerOrUrl;
					const defaultVersion = "*";
					const isEntry = s => s.startsWith("http") || s.includes(MANIFEST_EXT);
					// Check if the string starts with a type
					if (strSplit.length >= 2) {
						let [name, ...versionOrEntryArr] = strSplit;
						// @name@manifest-url.json
						if (str.startsWith(separator)) {
							name = strSplit.slice(0, 2).join(separator);
							versionOrEntryArr = [
								devVersionOrUrl || strSplit.slice(2).join(separator)
							];
						}
						let versionOrEntry =
							devVersionOrUrl || versionOrEntryArr.join(separator);
						if (isEntry(versionOrEntry)) {
							return {
								name,
								entry: versionOrEntry
							};
						} else {
							// Apply version rule
							// devVersionOrUrl => inputVersion => defaultVersion
							return {
								name,
								version: versionOrEntry || defaultVersion
							};
						}
					} else if (strSplit.length === 1) {
						const [name] = strSplit;
						if (devVersionOrUrl && isEntry(devVersionOrUrl)) {
							return {
								name,
								entry: devVersionOrUrl
							};
						}
						return {
							name,
							version: devVersionOrUrl || defaultVersion
						};
					} else {
						throw `Invalid entry value: ${str}`;
					}
				};
				const composeKeyWithSeparator = function (...args) {
					if (!args.length) {
						return "";
					}
					return args.reduce((sum, cur) => {
						if (!cur) {
							return sum;
						}
						if (!sum) {
							return cur;
						}
						return `${sum}${SEPARATOR}${cur}`;
					}, "");
				};
				const encodeName = function (name, prefix = "", withExt = false) {
					try {
						const ext = withExt ? ".js" : "";
						return `${prefix}${name
							.replace(
								new RegExp(`${NameTransformSymbol.AT}`, "g"),
								NameTransformMap[NameTransformSymbol.AT]
							)
							.replace(
								new RegExp(`${NameTransformSymbol.HYPHEN}`, "g"),
								NameTransformMap[NameTransformSymbol.HYPHEN]
							)
							.replace(
								new RegExp(`${NameTransformSymbol.SLASH}`, "g"),
								NameTransformMap[NameTransformSymbol.SLASH]
							)}${ext}`;
					} catch (err) {
						throw err;
					}
				};
				const decodeName = function (name, prefix, withExt) {
					try {
						let decodedName = name;
						if (prefix) {
							if (!decodedName.startsWith(prefix)) {
								return decodedName;
							}
							decodedName = decodedName.replace(new RegExp(prefix, "g"), "");
						}
						decodedName = decodedName
							.replace(
								new RegExp(`${NameTransformMap[NameTransformSymbol.AT]}`, "g"),
								EncodedNameTransformMap[
									NameTransformMap[NameTransformSymbol.AT]
								]
							)
							.replace(
								new RegExp(
									`${NameTransformMap[NameTransformSymbol.SLASH]}`,
									"g"
								),
								EncodedNameTransformMap[
									NameTransformMap[NameTransformSymbol.SLASH]
								]
							)
							.replace(
								new RegExp(
									`${NameTransformMap[NameTransformSymbol.HYPHEN]}`,
									"g"
								),
								EncodedNameTransformMap[
									NameTransformMap[NameTransformSymbol.HYPHEN]
								]
							);
						if (withExt) {
							decodedName = decodedName.replace(".js", "");
						}
						return decodedName;
					} catch (err) {
						throw err;
					}
				};
				const generateExposeFilename = (exposeName, withExt) => {
					if (!exposeName) {
						return "";
					}
					let expose = exposeName;
					if (expose === ".") {
						expose = "default_export";
					}
					if (expose.startsWith("./")) {
						expose = expose.replace("./", "");
					}
					return encodeName(expose, "__federation_expose_", withExt);
				};
				const generateShareFilename = (pkgName, withExt) => {
					if (!pkgName) {
						return "";
					}
					return encodeName(pkgName, "__federation_shared_", withExt);
				};
				const getResourceUrl = (module, sourceUrl) => {
					if ("getPublicPath" in module) {
						let publicPath;
						if (!module.getPublicPath.startsWith("function")) {
							publicPath = new Function(module.getPublicPath)();
						} else {
							publicPath = new Function("return " + module.getPublicPath)()();
						}
						return `${publicPath}${sourceUrl}`;
					} else if ("publicPath" in module) {
						if (
							!isBrowserEnv() &&
							!isReactNativeEnv() &&
							"ssrPublicPath" in module
						) {
							return `${module.ssrPublicPath}${sourceUrl}`;
						}
						return `${module.publicPath}${sourceUrl}`;
					} else {
						console.warn(
							"Cannot get resource URL. If in debug mode, please ignore.",
							module,
							sourceUrl
						);
						return "";
					}
				};
				// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
				const assert = (condition, msg) => {
					if (!condition) {
						error(msg);
					}
				};
				const error = msg => {
					throw new Error(`${LOG_CATEGORY}: ${msg}`);
				};
				const warn = msg => {
					console.warn(`${LOG_CATEGORY}: ${msg}`);
				};
				function safeToString(info) {
					try {
						return JSON.stringify(info, null, 2);
					} catch (e) {
						return "";
					}
				}
				// RegExp for version string
				const VERSION_PATTERN_REGEXP = /^([\d^=v<>~]|[*xX]$)/;
				function isRequiredVersion(str) {
					return VERSION_PATTERN_REGEXP.test(str);
				}

				const simpleJoinRemoteEntry = (rPath, rName) => {
					if (!rPath) {
						return rName;
					}
					const transformPath = str => {
						if (str === ".") {
							return "";
						}
						if (str.startsWith("./")) {
							return str.replace("./", "");
						}
						if (str.startsWith("/")) {
							const strWithoutSlash = str.slice(1);
							if (strWithoutSlash.endsWith("/")) {
								return strWithoutSlash.slice(0, -1);
							}
							return strWithoutSlash;
						}
						return str;
					};
					const transformedPath = transformPath(rPath);
					if (!transformedPath) {
						return rName;
					}
					if (transformedPath.endsWith("/")) {
						return `${transformedPath}${rName}`;
					}
					return `${transformedPath}/${rName}`;
				};
				function inferAutoPublicPath(url) {
					return url
						.replace(/#.*$/, "")
						.replace(/\?.*$/, "")
						.replace(/\/[^\/]+$/, "/");
				}
				// Priority: overrides > remotes
				// eslint-disable-next-line max-lines-per-function
				function generateSnapshotFromManifest(manifest, options = {}) {
					var _manifest_metaData, _manifest_metaData1;
					const { remotes = {}, overrides = {}, version } = options;
					let remoteSnapshot;
					const getPublicPath = () => {
						if ("publicPath" in manifest.metaData) {
							if (manifest.metaData.publicPath === "auto" && version) {
								// use same implementation as publicPath auto runtime module implements
								return inferAutoPublicPath(version);
							}
							return manifest.metaData.publicPath;
						} else {
							return manifest.metaData.getPublicPath;
						}
					};
					const overridesKeys = Object.keys(overrides);
					let remotesInfo = {};
					// If remotes are not provided, only the remotes in the manifest will be read
					if (!Object.keys(remotes).length) {
						var _manifest_remotes;
						remotesInfo =
							((_manifest_remotes = manifest.remotes) == null
								? void 0
								: _manifest_remotes.reduce((res, next) => {
										let matchedVersion;
										const name = next.federationContainerName;
										// overrides have higher priority
										if (overridesKeys.includes(name)) {
											matchedVersion = overrides[name];
										} else {
											if ("version" in next) {
												matchedVersion = next.version;
											} else {
												matchedVersion = next.entry;
											}
										}
										res[name] = {
											matchedVersion
										};
										return res;
									}, {})) || {};
					}
					// If remotes (deploy scenario) are specified, they need to be traversed again
					Object.keys(remotes).forEach(
						key =>
							(remotesInfo[key] = {
								// overrides will override dependencies
								matchedVersion: overridesKeys.includes(key)
									? overrides[key]
									: remotes[key]
							})
					);
					const {
						remoteEntry: {
							path: remoteEntryPath,
							name: remoteEntryName,
							type: remoteEntryType
						},
						types: remoteTypes,
						buildInfo: { buildVersion },
						globalName,
						ssrRemoteEntry
					} = manifest.metaData;
					const { exposes } = manifest;
					let basicRemoteSnapshot = {
						version: version ? version : "",
						buildVersion,
						globalName,
						remoteEntry: simpleJoinRemoteEntry(
							remoteEntryPath,
							remoteEntryName
						),
						remoteEntryType,
						remoteTypes: simpleJoinRemoteEntry(
							remoteTypes.path,
							remoteTypes.name
						),
						remoteTypesZip: remoteTypes.zip || "",
						remoteTypesAPI: remoteTypes.api || "",
						remotesInfo,
						shared:
							manifest == null
								? void 0
								: manifest.shared.map(item => ({
										assets: item.assets,
										sharedName: item.name,
										version: item.version
									})),
						modules:
							exposes == null
								? void 0
								: exposes.map(expose => ({
										moduleName: expose.name,
										modulePath: expose.path,
										assets: expose.assets
									}))
					};
					if (
						(_manifest_metaData = manifest.metaData) == null
							? void 0
							: _manifest_metaData.prefetchInterface
					) {
						const prefetchInterface = manifest.metaData.prefetchInterface;
						basicRemoteSnapshot = polyfills._({}, basicRemoteSnapshot, {
							prefetchInterface
						});
					}
					if (
						(_manifest_metaData1 = manifest.metaData) == null
							? void 0
							: _manifest_metaData1.prefetchEntry
					) {
						const { path, name, type } = manifest.metaData.prefetchEntry;
						basicRemoteSnapshot = polyfills._({}, basicRemoteSnapshot, {
							prefetchEntry: simpleJoinRemoteEntry(path, name),
							prefetchEntryType: type
						});
					}
					if ("publicPath" in manifest.metaData) {
						remoteSnapshot = polyfills._({}, basicRemoteSnapshot, {
							publicPath: getPublicPath(),
							ssrPublicPath: manifest.metaData.ssrPublicPath
						});
					} else {
						remoteSnapshot = polyfills._({}, basicRemoteSnapshot, {
							getPublicPath: getPublicPath()
						});
					}
					if (ssrRemoteEntry) {
						const fullSSRRemoteEntry = simpleJoinRemoteEntry(
							ssrRemoteEntry.path,
							ssrRemoteEntry.name
						);
						remoteSnapshot.ssrRemoteEntry = fullSSRRemoteEntry;
						remoteSnapshot.ssrRemoteEntryType =
							ssrRemoteEntry.type || "commonjs-module";
					}
					return remoteSnapshot;
				}
				function isManifestProvider(moduleInfo) {
					if (
						"remoteEntry" in moduleInfo &&
						moduleInfo.remoteEntry.includes(MANIFEST_EXT)
					) {
						return true;
					} else {
						return false;
					}
				}

				const PREFIX = "[ Module Federation ]";
				let Logger = class Logger {
					setPrefix(prefix) {
						this.prefix = prefix;
					}
					log(...args) {
						console.log(this.prefix, ...args);
					}
					warn(...args) {
						console.log(this.prefix, ...args);
					}
					error(...args) {
						console.log(this.prefix, ...args);
					}
					success(...args) {
						console.log(this.prefix, ...args);
					}
					info(...args) {
						console.log(this.prefix, ...args);
					}
					ready(...args) {
						console.log(this.prefix, ...args);
					}
					debug(...args) {
						if (isDebugMode()) {
							console.log(this.prefix, ...args);
						}
					}
					constructor(prefix) {
						this.prefix = prefix;
					}
				};
				function createLogger(prefix) {
					return new Logger(prefix);
				}
				const logger = createLogger(PREFIX);

				// eslint-disable-next-line @typescript-eslint/no-explicit-any
				async function safeWrapper(callback, disableWarn) {
					try {
						const res = await callback();
						return res;
					} catch (e) {
						!disableWarn && warn(e);
						return;
					}
				}
				function isStaticResourcesEqual(url1, url2) {
					const REG_EXP = /^(https?:)?\/\//i;
					// Transform url1 and url2 into relative paths
					const relativeUrl1 = url1.replace(REG_EXP, "").replace(/\/$/, "");
					const relativeUrl2 = url2.replace(REG_EXP, "").replace(/\/$/, "");
					// Check if the relative paths are identical
					return relativeUrl1 === relativeUrl2;
				}
				function createScript(info) {
					// Retrieve the existing script element by its src attribute
					let script = null;
					let needAttach = true;
					let timeout = 20000;
					let timeoutId;
					const scripts = document.getElementsByTagName("script");
					for (let i = 0; i < scripts.length; i++) {
						const s = scripts[i];
						const scriptSrc = s.getAttribute("src");
						if (scriptSrc && isStaticResourcesEqual(scriptSrc, info.url)) {
							script = s;
							needAttach = false;
							break;
						}
					}
					if (!script) {
						const attrs = info.attrs;
						script = document.createElement("script");
						script.type =
							(attrs == null ? void 0 : attrs["type"]) === "module"
								? "module"
								: "text/javascript";
						let createScriptRes = undefined;
						if (info.createScriptHook) {
							createScriptRes = info.createScriptHook(info.url, info.attrs);
							if (createScriptRes instanceof HTMLScriptElement) {
								script = createScriptRes;
							} else if (typeof createScriptRes === "object") {
								if ("script" in createScriptRes && createScriptRes.script) {
									script = createScriptRes.script;
								}
								if ("timeout" in createScriptRes && createScriptRes.timeout) {
									timeout = createScriptRes.timeout;
								}
							}
						}
						if (!script.src) {
							script.src = info.url;
						}
						if (attrs && !createScriptRes) {
							Object.keys(attrs).forEach(name => {
								if (script) {
									if (name === "async" || name === "defer") {
										script[name] = attrs[name];
										// Attributes that do not exist are considered overridden
									} else if (!script.getAttribute(name)) {
										script.setAttribute(name, attrs[name]);
									}
								}
							});
						}
					}
					const onScriptComplete = async (
						prev, // eslint-disable-next-line @typescript-eslint/no-explicit-any
						event
					) => {
						clearTimeout(timeoutId);
						const onScriptCompleteCallback = () => {
							if ((event == null ? void 0 : event.type) === "error") {
								(info == null ? void 0 : info.onErrorCallback) &&
									(info == null ? void 0 : info.onErrorCallback(event));
							} else {
								(info == null ? void 0 : info.cb) &&
									(info == null ? void 0 : info.cb());
							}
						};
						// Prevent memory leaks in IE.
						if (script) {
							script.onerror = null;
							script.onload = null;
							safeWrapper(() => {
								const { needDeleteScript = true } = info;
								if (needDeleteScript) {
									(script == null ? void 0 : script.parentNode) &&
										script.parentNode.removeChild(script);
								}
							});
							if (prev && typeof prev === "function") {
								const result = prev(event);
								if (result instanceof Promise) {
									const res = await result;
									onScriptCompleteCallback();
									return res;
								}
								onScriptCompleteCallback();
								return result;
							}
						}
						onScriptCompleteCallback();
					};
					script.onerror = onScriptComplete.bind(null, script.onerror);
					script.onload = onScriptComplete.bind(null, script.onload);
					timeoutId = setTimeout(() => {
						onScriptComplete(
							null,
							new Error(`Remote script "${info.url}" time-outed.`)
						);
					}, timeout);
					return {
						script,
						needAttach
					};
				}
				function createLink(info) {
					// <link rel="preload" href="script.js" as="script">
					// Retrieve the existing script element by its src attribute
					let link = null;
					let needAttach = true;
					const links = document.getElementsByTagName("link");
					for (let i = 0; i < links.length; i++) {
						const l = links[i];
						const linkHref = l.getAttribute("href");
						const linkRel = l.getAttribute("rel");
						if (
							linkHref &&
							isStaticResourcesEqual(linkHref, info.url) &&
							linkRel === info.attrs["rel"]
						) {
							link = l;
							needAttach = false;
							break;
						}
					}
					if (!link) {
						link = document.createElement("link");
						link.setAttribute("href", info.url);
						let createLinkRes = undefined;
						const attrs = info.attrs;
						if (info.createLinkHook) {
							createLinkRes = info.createLinkHook(info.url, attrs);
							if (createLinkRes instanceof HTMLLinkElement) {
								link = createLinkRes;
							}
						}
						if (attrs && !createLinkRes) {
							Object.keys(attrs).forEach(name => {
								if (link && !link.getAttribute(name)) {
									link.setAttribute(name, attrs[name]);
								}
							});
						}
					}
					const onLinkComplete = (
						prev, // eslint-disable-next-line @typescript-eslint/no-explicit-any
						event
					) => {
						const onLinkCompleteCallback = () => {
							if ((event == null ? void 0 : event.type) === "error") {
								(info == null ? void 0 : info.onErrorCallback) &&
									(info == null ? void 0 : info.onErrorCallback(event));
							} else {
								(info == null ? void 0 : info.cb) &&
									(info == null ? void 0 : info.cb());
							}
						};
						// Prevent memory leaks in IE.
						if (link) {
							link.onerror = null;
							link.onload = null;
							safeWrapper(() => {
								const { needDeleteLink = true } = info;
								if (needDeleteLink) {
									(link == null ? void 0 : link.parentNode) &&
										link.parentNode.removeChild(link);
								}
							});
							if (prev) {
								// eslint-disable-next-line @typescript-eslint/no-explicit-any
								const res = prev(event);
								onLinkCompleteCallback();
								return res;
							}
						}
						onLinkCompleteCallback();
					};
					link.onerror = onLinkComplete.bind(null, link.onerror);
					link.onload = onLinkComplete.bind(null, link.onload);
					return {
						link,
						needAttach
					};
				}
				function loadScript(url, info) {
					const { attrs = {}, createScriptHook } = info;
					return new Promise((resolve, reject) => {
						const { script, needAttach } = createScript({
							url,
							cb: resolve,
							onErrorCallback: reject,
							attrs: polyfills._(
								{
									fetchpriority: "high"
								},
								attrs
							),
							createScriptHook,
							needDeleteScript: true
						});
						needAttach && document.head.appendChild(script);
					});
				}

				function importNodeModule(name) {
					if (!name) {
						throw new Error("import specifier is required");
					}
					const importModule = new Function("name", `return import(name)`);
					return importModule(name)
						.then(res => res)
						.catch(error => {
							console.error(`Error importing module ${name}:`, error);
							throw error;
						});
				}
				const loadNodeFetch = async () => {
					const fetchModule = await importNodeModule("node-fetch");
					return fetchModule.default || fetchModule;
				};
				const lazyLoaderHookFetch = async (input, init, loaderHook) => {
					const hook = (url, init) => {
						return loaderHook.lifecycle.fetch.emit(url, init);
					};
					const res = await hook(input, init || {});
					if (!res || !(res instanceof Response)) {
						const fetchFunction =
							typeof fetch === "undefined" ? await loadNodeFetch() : fetch;
						return fetchFunction(input, init || {});
					}
					return res;
				};
				const createScriptNode =
					typeof ENV_TARGET === "undefined" || ENV_TARGET !== "web"
						? (url, cb, attrs, loaderHook) => {
								if (loaderHook == null ? void 0 : loaderHook.createScriptHook) {
									const hookResult = loaderHook.createScriptHook(url);
									if (
										hookResult &&
										typeof hookResult === "object" &&
										"url" in hookResult
									) {
										url = hookResult.url;
									}
								}
								let urlObj;
								try {
									urlObj = new URL(url);
								} catch (e) {
									console.error("Error constructing URL:", e);
									cb(new Error(`Invalid URL: ${e}`));
									return;
								}
								const getFetch = async () => {
									if (loaderHook == null ? void 0 : loaderHook.fetch) {
										return (input, init) =>
											lazyLoaderHookFetch(input, init, loaderHook);
									}
									return typeof fetch === "undefined" ? loadNodeFetch() : fetch;
								};
								const handleScriptFetch = async (f, urlObj) => {
									try {
										var //@ts-ignore
											_vm_constants;
										const res = await f(urlObj.href);
										const data = await res.text();
										const [path, vm] = await Promise.all([
											importNodeModule("path"),
											importNodeModule("vm")
										]);
										const scriptContext = {
											exports: {},
											module: {
												exports: {}
											}
										};
										const urlDirname = urlObj.pathname
											.split("/")
											.slice(0, -1)
											.join("/");
										const filename = path.basename(urlObj.pathname);
										var _vm_constants_USE_MAIN_CONTEXT_DEFAULT_LOADER;
										const script = new vm.Script(
											`(function(exports, module, require, __dirname, __filename) {${data}\n})`,
											{
												filename,
												importModuleDynamically:
													(_vm_constants_USE_MAIN_CONTEXT_DEFAULT_LOADER =
														(_vm_constants = vm.constants) == null
															? void 0
															: _vm_constants.USE_MAIN_CONTEXT_DEFAULT_LOADER) !=
													null
														? _vm_constants_USE_MAIN_CONTEXT_DEFAULT_LOADER
														: importNodeModule
											}
										);
										script.runInThisContext()(
											scriptContext.exports,
											scriptContext.module,
											eval("require"),
											urlDirname,
											filename
										);
										const exportedInterface =
											scriptContext.module.exports || scriptContext.exports;
										if (attrs && exportedInterface && attrs["globalName"]) {
											const container =
												exportedInterface[attrs["globalName"]] ||
												exportedInterface;
											cb(undefined, container);
											return;
										}
										cb(undefined, exportedInterface);
									} catch (e) {
										cb(
											e instanceof Error
												? e
												: new Error(`Script execution error: ${e}`)
										);
									}
								};
								getFetch()
									.then(async f => {
										if (
											(attrs == null ? void 0 : attrs["type"]) === "esm" ||
											(attrs == null ? void 0 : attrs["type"]) === "module"
										) {
											return loadModule(urlObj.href, {
												fetch: f,
												vm: await importNodeModule("vm")
											})
												.then(async module => {
													await module.evaluate();
													cb(undefined, module.namespace);
												})
												.catch(e => {
													cb(
														e instanceof Error
															? e
															: new Error(`Script execution error: ${e}`)
													);
												});
										}
										handleScriptFetch(f, urlObj);
									})
									.catch(err => {
										cb(err);
									});
							}
						: (url, cb, attrs, loaderHook) => {
								cb(
									new Error(
										"createScriptNode is disabled in non-Node.js environment"
									)
								);
							};
				const loadScriptNode =
					typeof ENV_TARGET === "undefined" || ENV_TARGET !== "web"
						? (url, info) => {
								return new Promise((resolve, reject) => {
									createScriptNode(
										url,
										(error, scriptContext) => {
											if (error) {
												reject(error);
											} else {
												var _info_attrs, _info_attrs1;
												const remoteEntryKey =
													(info == null
														? void 0
														: (_info_attrs = info.attrs) == null
															? void 0
															: _info_attrs["globalName"]) ||
													`__FEDERATION_${info == null ? void 0 : (_info_attrs1 = info.attrs) == null ? void 0 : _info_attrs1["name"]}:custom__`;
												const entryExports = (globalThis[remoteEntryKey] =
													scriptContext);
												resolve(entryExports);
											}
										},
										info.attrs,
										info.loaderHook
									);
								});
							}
						: (url, info) => {
								throw new Error(
									"loadScriptNode is disabled in non-Node.js environment"
								);
							};
				async function loadModule(url, options) {
					const { fetch: fetch1, vm } = options;
					const response = await fetch1(url);
					const code = await response.text();
					const module = new vm.SourceTextModule(code, {
						// @ts-ignore
						importModuleDynamically: async (specifier, script) => {
							const resolvedUrl = new URL(specifier, url).href;
							return loadModule(resolvedUrl, options);
						}
					});
					await module.link(async specifier => {
						const resolvedUrl = new URL(specifier, url).href;
						const module = await loadModule(resolvedUrl, options);
						return module;
					});
					return module;
				}

				function normalizeOptions(enableDefault, defaultOptions, key) {
					return function (options) {
						if (options === false) {
							return false;
						}
						if (typeof options === "undefined") {
							if (enableDefault) {
								return defaultOptions;
							} else {
								return false;
							}
						}
						if (options === true) {
							return defaultOptions;
						}
						if (options && typeof options === "object") {
							return polyfills._({}, defaultOptions, options);
						}
						throw new Error(
							`Unexpected type for \`${key}\`, expect boolean/undefined/object, got: ${typeof options}`
						);
					};
				}

				const createModuleFederationConfig = options => {
					return options;
				};

				exports.BROWSER_LOG_KEY = BROWSER_LOG_KEY;
				exports.ENCODE_NAME_PREFIX = ENCODE_NAME_PREFIX;
				exports.EncodedNameTransformMap = EncodedNameTransformMap;
				exports.FederationModuleManifest = FederationModuleManifest;
				exports.MANIFEST_EXT = MANIFEST_EXT;
				exports.MFModuleType = MFModuleType;
				exports.MFPrefetchCommon = MFPrefetchCommon;
				exports.MODULE_DEVTOOL_IDENTIFIER = MODULE_DEVTOOL_IDENTIFIER;
				exports.ManifestFileName = ManifestFileName;
				exports.NameTransformMap = NameTransformMap;
				exports.NameTransformSymbol = NameTransformSymbol;
				exports.SEPARATOR = SEPARATOR;
				exports.StatsFileName = StatsFileName;
				exports.TEMP_DIR = TEMP_DIR;
				exports.assert = assert;
				exports.composeKeyWithSeparator = composeKeyWithSeparator;
				exports.containerPlugin = ContainerPlugin;
				exports.containerReferencePlugin = ContainerReferencePlugin;
				exports.createLink = createLink;
				exports.createLogger = createLogger;
				exports.createModuleFederationConfig = createModuleFederationConfig;
				exports.createScript = createScript;
				exports.createScriptNode = createScriptNode;
				exports.decodeName = decodeName;
				exports.encodeName = encodeName;
				exports.error = error;
				exports.generateExposeFilename = generateExposeFilename;
				exports.generateShareFilename = generateShareFilename;
				exports.generateSnapshotFromManifest = generateSnapshotFromManifest;
				exports.getProcessEnv = getProcessEnv;
				exports.getResourceUrl = getResourceUrl;
				exports.inferAutoPublicPath = inferAutoPublicPath;
				exports.isBrowserEnv = isBrowserEnv;
				exports.isDebugMode = isDebugMode;
				exports.isManifestProvider = isManifestProvider;
				exports.isReactNativeEnv = isReactNativeEnv;
				exports.isRequiredVersion = isRequiredVersion;
				exports.isStaticResourcesEqual = isStaticResourcesEqual;
				exports.loadScript = loadScript;
				exports.loadScriptNode = loadScriptNode;
				exports.logger = logger;
				exports.moduleFederationPlugin = ModuleFederationPlugin;
				exports.normalizeOptions = normalizeOptions;
				exports.parseEntry = parseEntry;
				exports.safeToString = safeToString;
				exports.safeWrapper = safeWrapper;
				exports.sharePlugin = SharePlugin;
				exports.simpleJoinRemoteEntry = simpleJoinRemoteEntry;
				exports.warn = warn;
			},
		"../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/polyfills.cjs.cjs":
			/*!*************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/polyfills.cjs.cjs ***!
  \*************************************************************************************************************************/
			function (__unused_webpack_module, exports) {
				function _extends() {
					_extends =
						Object.assign ||
						function assign(target) {
							for (var i = 1; i < arguments.length; i++) {
								var source = arguments[i];
								for (var key in source)
									if (Object.prototype.hasOwnProperty.call(source, key))
										target[key] = source[key];
							}
							return target;
						};
					return _extends.apply(this, arguments);
				}

				exports._ = _extends;
			},
		"../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/constant.cjs.cjs":
			/*!****************************************************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/constant.cjs.cjs ***!
  \****************************************************************************************************************************************************************/
			function (__unused_webpack_module, exports) {
				const FEDERATION_SUPPORTED_TYPES = ["script"];

				exports.FEDERATION_SUPPORTED_TYPES = FEDERATION_SUPPORTED_TYPES;
			},
		"../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs":
			/*!*************************************************************************************************************************************************************!*\
  !*** ../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs ***!
  \*************************************************************************************************************************************************************/
			function (module, __unused_webpack_exports, __webpack_require__) {
				var runtime = __webpack_require__(
					/*! @module-federation/runtime */ "../../node_modules/.pnpm/@module-federation+runtime@0.16.0/node_modules/@module-federation/runtime/dist/index.cjs.cjs"
				);
				var constant = __webpack_require__(
					/*! ./constant.cjs.cjs */ "../../node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/constant.cjs.cjs"
				);
				var sdk = __webpack_require__(
					/*! @module-federation/sdk */ "../../node_modules/.pnpm/@module-federation+sdk@0.16.0/node_modules/@module-federation/sdk/dist/index.cjs.cjs"
				);

				function _interopNamespaceDefault(e) {
					var n = Object.create(null);
					if (e) {
						for (var k in e) {
							n[k] = e[k];
						}
					}
					n.default = e;
					return Object.freeze(n);
				}

				var runtime__namespace =
					/*#__PURE__*/ _interopNamespaceDefault(runtime);

				function attachShareScopeMap(webpackRequire) {
					if (
						!webpackRequire.S ||
						webpackRequire.federation.hasAttachShareScopeMap ||
						!webpackRequire.federation.instance ||
						!webpackRequire.federation.instance.shareScopeMap
					) {
						return;
					}
					webpackRequire.S = webpackRequire.federation.instance.shareScopeMap;
					webpackRequire.federation.hasAttachShareScopeMap = true;
				}

				function remotes(options) {
					const {
						chunkId,
						promises,
						chunkMapping,
						idToExternalAndNameMapping,
						webpackRequire,
						idToRemoteMap
					} = options;
					attachShareScopeMap(webpackRequire);
					if (webpackRequire.o(chunkMapping, chunkId)) {
						chunkMapping[chunkId].forEach(id => {
							let getScope = webpackRequire.R;
							if (!getScope) {
								getScope = [];
							}
							const data = idToExternalAndNameMapping[id];
							const remoteInfos = idToRemoteMap[id];
							// @ts-ignore seems not work
							if (getScope.indexOf(data) >= 0) {
								return;
							}
							// @ts-ignore seems not work
							getScope.push(data);
							if (data.p) {
								return promises.push(data.p);
							}
							const onError = error => {
								if (!error) {
									error = new Error("Container missing");
								}
								if (typeof error.message === "string") {
									error.message += `\nwhile loading "${data[1]}" from ${data[2]}`;
								}
								webpackRequire.m[id] = () => {
									throw error;
								};
								data.p = 0;
							};
							const handleFunction = (fn, arg1, arg2, d, next, first) => {
								try {
									const promise = fn(arg1, arg2);
									if (promise && promise.then) {
										const p = promise.then(result => next(result, d), onError);
										if (first) {
											promises.push((data.p = p));
										} else {
											return p;
										}
									} else {
										return next(promise, d, first);
									}
								} catch (error) {
									onError(error);
								}
							};
							const onExternal = (external, _, first) =>
								external
									? handleFunction(
											webpackRequire.I,
											data[0],
											0,
											external,
											onInitialized,
											first
										)
									: onError();
							// eslint-disable-next-line no-var
							var onInitialized = (_, external, first) =>
								handleFunction(
									external.get,
									data[1],
									getScope,
									0,
									onFactory,
									first
								);
							// eslint-disable-next-line no-var
							var onFactory = factory => {
								data.p = 1;
								webpackRequire.m[id] = module => {
									module.exports = factory();
								};
							};
							const onRemoteLoaded = () => {
								try {
									const remoteName = sdk.decodeName(
										remoteInfos[0].name,
										sdk.ENCODE_NAME_PREFIX
									);
									const remoteModuleName = remoteName + data[1].slice(1);
									const instance = webpackRequire.federation.instance;
									const loadRemote = () =>
										webpackRequire.federation.instance.loadRemote(
											remoteModuleName,
											{
												loadFactory: false,
												from: "build"
											}
										);
									if (instance.options.shareStrategy === "version-first") {
										return Promise.all(
											instance.sharedHandler.initializeSharing(data[0])
										).then(() => {
											return loadRemote();
										});
									}
									return loadRemote();
								} catch (error) {
									onError(error);
								}
							};
							const useRuntimeLoad =
								remoteInfos.length === 1 &&
								constant.FEDERATION_SUPPORTED_TYPES.includes(
									remoteInfos[0].externalType
								) &&
								remoteInfos[0].name;
							if (useRuntimeLoad) {
								handleFunction(onRemoteLoaded, data[2], 0, 0, onFactory, 1);
							} else {
								handleFunction(webpackRequire, data[2], 0, 0, onExternal, 1);
							}
						});
					}
				}

				function consumes(options) {
					const {
						chunkId,
						promises,
						chunkMapping,
						installedModules,
						moduleToHandlerMapping,
						webpackRequire
					} = options;
					attachShareScopeMap(webpackRequire);
					if (webpackRequire.o(chunkMapping, chunkId)) {
						chunkMapping[chunkId].forEach(id => {
							if (webpackRequire.o(installedModules, id)) {
								return promises.push(installedModules[id]);
							}
							const onFactory = factory => {
								installedModules[id] = 0;
								webpackRequire.m[id] = module => {
									delete webpackRequire.c[id];
									module.exports = factory();
								};
							};
							const onError = error => {
								delete installedModules[id];
								webpackRequire.m[id] = module => {
									delete webpackRequire.c[id];
									throw error;
								};
							};
							try {
								const federationInstance = webpackRequire.federation.instance;
								if (!federationInstance) {
									throw new Error("Federation instance not found!");
								}
								const { shareKey, getter, shareInfo } =
									moduleToHandlerMapping[id];
								const promise = federationInstance
									.loadShare(shareKey, {
										customShareInfo: shareInfo
									})
									.then(factory => {
										if (factory === false) {
											return getter();
										}
										return factory;
									});
								if (promise.then) {
									promises.push(
										(installedModules[id] = promise
											.then(onFactory)
											.catch(onError))
									);
								} else {
									// @ts-ignore maintain previous logic
									onFactory(promise);
								}
							} catch (e) {
								onError(e);
							}
						});
					}
				}

				function initializeSharing({
					shareScopeName,
					webpackRequire,
					initPromises,
					initTokens,
					initScope
				}) {
					const shareScopeKeys = Array.isArray(shareScopeName)
						? shareScopeName
						: [shareScopeName];
					var initializeSharingPromises = [];
					var _initializeSharing = function (shareScopeKey) {
						if (!initScope) initScope = [];
						const mfInstance = webpackRequire.federation.instance;
						// handling circular init calls
						var initToken = initTokens[shareScopeKey];
						if (!initToken)
							initToken = initTokens[shareScopeKey] = {
								from: mfInstance.name
							};
						if (initScope.indexOf(initToken) >= 0) return;
						initScope.push(initToken);
						const promise = initPromises[shareScopeKey];
						if (promise) return promise;
						var warn = msg =>
							typeof console !== "undefined" &&
							console.warn &&
							console.warn(msg);
						var initExternal = id => {
							var handleError = err =>
								warn("Initialization of sharing external failed: " + err);
							try {
								var module = webpackRequire(id);
								if (!module) return;
								var initFn = module =>
									module &&
									module.init && // @ts-ignore compat legacy mf shared behavior
									module.init(webpackRequire.S[shareScopeKey], initScope, {
										shareScopeMap: webpackRequire.S || {},
										shareScopeKeys: shareScopeName
									});
								if (module.then)
									return promises.push(module.then(initFn, handleError));
								var initResult = initFn(module);
								// @ts-ignore
								if (
									initResult &&
									typeof initResult !== "boolean" &&
									initResult.then
								)
									// @ts-ignore
									return promises.push(initResult["catch"](handleError));
							} catch (err) {
								handleError(err);
							}
						};
						const promises = mfInstance.initializeSharing(shareScopeKey, {
							strategy: mfInstance.options.shareStrategy,
							initScope,
							from: "build"
						});
						attachShareScopeMap(webpackRequire);
						const bundlerRuntimeRemotesOptions =
							webpackRequire.federation.bundlerRuntimeOptions.remotes;
						if (bundlerRuntimeRemotesOptions) {
							Object.keys(bundlerRuntimeRemotesOptions.idToRemoteMap).forEach(
								moduleId => {
									const info =
										bundlerRuntimeRemotesOptions.idToRemoteMap[moduleId];
									const externalModuleId =
										bundlerRuntimeRemotesOptions.idToExternalAndNameMapping[
											moduleId
										][2];
									if (info.length > 1) {
										initExternal(externalModuleId);
									} else if (info.length === 1) {
										const remoteInfo = info[0];
										if (
											!constant.FEDERATION_SUPPORTED_TYPES.includes(
												remoteInfo.externalType
											)
										) {
											initExternal(externalModuleId);
										}
									}
								}
							);
						}
						if (!promises.length) {
							return (initPromises[shareScopeKey] = true);
						}
						return (initPromises[shareScopeKey] = Promise.all(promises).then(
							() => (initPromises[shareScopeKey] = true)
						));
					};
					shareScopeKeys.forEach(key => {
						initializeSharingPromises.push(_initializeSharing(key));
					});
					return Promise.all(initializeSharingPromises).then(() => true);
				}

				function handleInitialConsumes(options) {
					const { moduleId, moduleToHandlerMapping, webpackRequire } = options;
					const federationInstance = webpackRequire.federation.instance;
					if (!federationInstance) {
						throw new Error("Federation instance not found!");
					}
					const { shareKey, shareInfo } = moduleToHandlerMapping[moduleId];
					try {
						return federationInstance.loadShareSync(shareKey, {
							customShareInfo: shareInfo
						});
					} catch (err) {
						console.error(
							'loadShareSync failed! The function should not be called unless you set "eager:true". If you do not set it, and encounter this issue, you can check whether an async boundary is implemented.'
						);
						console.error("The original error message is as follows: ");
						throw err;
					}
				}
				function installInitialConsumes(options) {
					const {
						moduleToHandlerMapping,
						webpackRequire,
						installedModules,
						initialConsumes
					} = options;
					initialConsumes.forEach(id => {
						webpackRequire.m[id] = module => {
							// Handle scenario when module is used synchronously
							installedModules[id] = 0;
							delete webpackRequire.c[id];
							const factory = handleInitialConsumes({
								moduleId: id,
								moduleToHandlerMapping,
								webpackRequire
							});
							if (typeof factory !== "function") {
								throw new Error(
									`Shared module is not available for eager consumption: ${id}`
								);
							}
							module.exports = factory();
						};
					});
				}

				function _extends() {
					_extends =
						Object.assign ||
						function assign(target) {
							for (var i = 1; i < arguments.length; i++) {
								var source = arguments[i];
								for (var key in source)
									if (Object.prototype.hasOwnProperty.call(source, key))
										target[key] = source[key];
							}
							return target;
						};
					return _extends.apply(this, arguments);
				}

				function initContainerEntry(options) {
					const {
						webpackRequire,
						shareScope,
						initScope,
						shareScopeKey,
						remoteEntryInitOptions
					} = options;
					if (!webpackRequire.S) return;
					if (
						!webpackRequire.federation ||
						!webpackRequire.federation.instance ||
						!webpackRequire.federation.initOptions
					)
						return;
					const federationInstance = webpackRequire.federation.instance;
					federationInstance.initOptions(
						_extends(
							{
								name: webpackRequire.federation.initOptions.name,
								remotes: []
							},
							remoteEntryInitOptions
						)
					);
					const hostShareScopeKeys =
						remoteEntryInitOptions == null
							? void 0
							: remoteEntryInitOptions.shareScopeKeys;
					const hostShareScopeMap =
						remoteEntryInitOptions == null
							? void 0
							: remoteEntryInitOptions.shareScopeMap;
					// host: 'default' remote: 'default'  remote['default'] = hostShareScopeMap['default']
					// host: ['default', 'scope1'] remote: 'default'  remote['default'] = hostShareScopeMap['default']; remote['scope1'] = hostShareScopeMap['scop1']
					// host: 'default' remote: ['default','scope1']  remote['default'] = hostShareScopeMap['default']; remote['scope1'] = hostShareScopeMap['scope1'] = {}
					// host: ['scope1','default'] remote: ['scope1','scope2'] => remote['scope1'] = hostShareScopeMap['scope1']; remote['scope2'] = hostShareScopeMap['scope2'] = {};
					if (!shareScopeKey || typeof shareScopeKey === "string") {
						const key = shareScopeKey || "default";
						if (Array.isArray(hostShareScopeKeys)) {
							// const sc = hostShareScopeMap![key];
							// if (!sc) {
							//   throw new Error('shareScopeKey is not exist in hostShareScopeMap');
							// }
							// federationInstance.initShareScopeMap(key, sc, {
							//   hostShareScopeMap: remoteEntryInitOptions?.shareScopeMap || {},
							// });
							hostShareScopeKeys.forEach(hostKey => {
								if (!hostShareScopeMap[hostKey]) {
									hostShareScopeMap[hostKey] = {};
								}
								const sc = hostShareScopeMap[hostKey];
								federationInstance.initShareScopeMap(hostKey, sc, {
									hostShareScopeMap:
										(remoteEntryInitOptions == null
											? void 0
											: remoteEntryInitOptions.shareScopeMap) || {}
								});
							});
						} else {
							federationInstance.initShareScopeMap(key, shareScope, {
								hostShareScopeMap:
									(remoteEntryInitOptions == null
										? void 0
										: remoteEntryInitOptions.shareScopeMap) || {}
							});
						}
					} else {
						shareScopeKey.forEach(key => {
							if (!hostShareScopeKeys || !hostShareScopeMap) {
								federationInstance.initShareScopeMap(key, shareScope, {
									hostShareScopeMap:
										(remoteEntryInitOptions == null
											? void 0
											: remoteEntryInitOptions.shareScopeMap) || {}
								});
								return;
							}
							if (!hostShareScopeMap[key]) {
								hostShareScopeMap[key] = {};
							}
							const sc = hostShareScopeMap[key];
							federationInstance.initShareScopeMap(key, sc, {
								hostShareScopeMap:
									(remoteEntryInitOptions == null
										? void 0
										: remoteEntryInitOptions.shareScopeMap) || {}
							});
						});
					}
					if (webpackRequire.federation.attachShareScopeMap) {
						webpackRequire.federation.attachShareScopeMap(webpackRequire);
					}
					if (typeof webpackRequire.federation.prefetch === "function") {
						webpackRequire.federation.prefetch();
					}
					if (!Array.isArray(shareScopeKey)) {
						// @ts-ignore
						return webpackRequire.I(shareScopeKey || "default", initScope);
					}
					var proxyInitializeSharing = Boolean(
						webpackRequire.federation.initOptions.shared
					);
					if (proxyInitializeSharing) {
						// @ts-ignore
						return webpackRequire.I(shareScopeKey, initScope);
					}
					// @ts-ignore
					return Promise.all(
						shareScopeKey.map(key => {
							// @ts-ignore
							return webpackRequire.I(key, initScope);
						})
					).then(() => true);
				}

				const federation = {
					runtime: runtime__namespace,
					instance: undefined,
					initOptions: undefined,
					bundlerRuntime: {
						remotes,
						consumes,
						I: initializeSharing,
						S: {},
						installInitialConsumes,
						initContainerEntry
					},
					attachShareScopeMap,
					bundlerRuntimeOptions: {}
				};

				module.exports = federation;
			},
		"./threejs-app/index.js":
			/*!******************************!*\
  !*** ./threejs-app/index.js ***!
  \******************************/
			function (
				__unused_webpack___webpack_module__,
				__unused_webpack___webpack_exports__,
				__webpack_require__
			) {
				/* ESM import */ var _src_main_js__WEBPACK_IMPORTED_MODULE_0__ =
					__webpack_require__(/*! ./src/main.js */ "./threejs-app/src/main.js");
				/* ESM import */ var _src_main_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ./src/main.js */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var _src_main_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ./src/main.js */ "webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js"
					);
				/* ESM import */ var _src_main_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! ./src/main.js */ "webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js"
					);
				/* ESM import */ var _src_materials_js__WEBPACK_IMPORTED_MODULE_1__ =
					__webpack_require__(
						/*! ./src/materials.js */ "./threejs-app/src/materials.js"
					);
				/* ESM import */ var _src_geometries_js__WEBPACK_IMPORTED_MODULE_2__ =
					__webpack_require__(
						/*! ./src/geometries.js */ "./threejs-app/src/geometries.js"
					);

				console.log("Starting Three.js application...");

				// Initialize scene
				const { scene, camera, renderer, cube } = (0,
				_src_main_js__WEBPACK_IMPORTED_MODULE_0__.createScene)();

				// Create materials
				const materials = (0,
				_src_materials_js__WEBPACK_IMPORTED_MODULE_1__.createAdvancedMaterials)();
				const materialFactory =
					new _src_materials_js__WEBPACK_IMPORTED_MODULE_1__.MaterialFactory();

				// Create geometries
				const geometries = (0,
				_src_geometries_js__WEBPACK_IMPORTED_MODULE_2__.createGeometries)();

				// Use various Three.js features
				const group = new _src_main_js__WEBPACK_IMPORTED_MODULE_3__.Group();
				Object.entries(geometries).forEach(([name, geometry], index) => {
					const material =
						Object.values(materials)[index % Object.keys(materials).length];
					const mesh = new _src_main_js__WEBPACK_IMPORTED_MODULE_3__.Mesh(
						geometry,
						material
					);
					mesh.position.x = (index % 4) * 3 - 4.5;
					mesh.position.y = Math.floor(index / 4) * 3 - 3;
					group.add(mesh);
				});

				scene.add(group);

				// Test loader usage
				const gltfLoader =
					new _src_main_js__WEBPACK_IMPORTED_MODULE_4__.GLTFLoader();
				const dracoLoader =
					new _src_main_js__WEBPACK_IMPORTED_MODULE_5__.DRACOLoader();
				gltfLoader.setDRACOLoader(dracoLoader);

				console.log(
					"Three.js app initialized with",
					Object.keys(geometries).length,
					"geometries"
				);
			},
		"./threejs-app/src/geometries.js":
			/*!***************************************!*\
  !*** ./threejs-app/src/geometries.js ***!
  \***************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.d(__webpack_exports__, {
					createGeometries: () => createGeometries
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);

				function createGeometries() {
					return {
						box: new three__WEBPACK_IMPORTED_MODULE_0__.BoxGeometry(1, 1, 1),
						sphere: new three__WEBPACK_IMPORTED_MODULE_0__.SphereGeometry(
							1,
							32,
							32
						),
						cylinder: new three__WEBPACK_IMPORTED_MODULE_0__.CylinderGeometry(
							1,
							1,
							2,
							32
						),
						torus: new three__WEBPACK_IMPORTED_MODULE_0__.TorusGeometry(
							1,
							0.4,
							16,
							100
						),
						torusKnot: new three__WEBPACK_IMPORTED_MODULE_0__.TorusKnotGeometry(
							1,
							0.3,
							100,
							16
						),
						plane: new three__WEBPACK_IMPORTED_MODULE_0__.PlaneGeometry(5, 5),
						cone: new three__WEBPACK_IMPORTED_MODULE_0__.ConeGeometry(1, 2, 32),
						dodecahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.DodecahedronGeometry(1),
						icosahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.IcosahedronGeometry(1),
						octahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.OctahedronGeometry(1),
						tetrahedron:
							new three__WEBPACK_IMPORTED_MODULE_0__.TetrahedronGeometry(1)
					};
				}

				class GeometryProcessor {
					static merge(geometries) {
						const merged = new THREE.BufferGeometry();
						// Simplified merge logic
						return merged;
					}

					static optimize(geometry) {
						geometry.computeVertexNormals();
						geometry.computeBoundingBox();
						geometry.computeBoundingSphere();
						return geometry;
					}
				}
			},
		"./threejs-app/src/main.js":
			/*!*********************************!*\
  !*** ./threejs-app/src/main.js ***!
  \*********************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				__webpack_require__.d(__webpack_exports__, {
					createScene: () => createScene
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);
				/* ESM import */ var three_examples_jsm_controls_OrbitControls_js__WEBPACK_IMPORTED_MODULE_1__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/controls/OrbitControls.js */ "webpack/sharing/consume/default/three-orbit-controls/three/examples/jsm/controls/OrbitControls.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_GLTFLoader_js__WEBPACK_IMPORTED_MODULE_2__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/GLTFLoader.js */ "webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_DRACOLoader_js__WEBPACK_IMPORTED_MODULE_3__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/DRACOLoader.js */ "webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js"
					);
				/* ESM import */ var three_examples_jsm_loaders_RGBELoader_js__WEBPACK_IMPORTED_MODULE_4__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/loaders/RGBELoader.js */ "webpack/sharing/consume/default/three-rgbe-loader/three/examples/jsm/loaders/RGBELoader.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_EffectComposer_js__WEBPACK_IMPORTED_MODULE_5__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/EffectComposer.js */ "webpack/sharing/consume/default/three-effect-composer/three/examples/jsm/postprocessing/EffectComposer.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_RenderPass_js__WEBPACK_IMPORTED_MODULE_6__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/RenderPass.js */ "webpack/sharing/consume/default/three-render-pass/three/examples/jsm/postprocessing/RenderPass.js"
					);
				/* ESM import */ var three_examples_jsm_postprocessing_UnrealBloomPass_js__WEBPACK_IMPORTED_MODULE_7__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three/examples/jsm/postprocessing/UnrealBloomPass.js */ "webpack/sharing/consume/default/three-bloom-pass/three/examples/jsm/postprocessing/UnrealBloomPass.js"
					);

				console.log(
					"Three.js version:",
					three__WEBPACK_IMPORTED_MODULE_0__.REVISION
				);

				// Create a scene
				function createScene() {
					const scene = new three__WEBPACK_IMPORTED_MODULE_0__.Scene();
					const camera =
						new three__WEBPACK_IMPORTED_MODULE_0__.PerspectiveCamera(
							75,
							window.innerWidth / window.innerHeight,
							0.1,
							1000
						);
					const renderer =
						new three__WEBPACK_IMPORTED_MODULE_0__.WebGLRenderer();

					renderer.setSize(window.innerWidth, window.innerHeight);

					// Add lights
					const ambientLight =
						new three__WEBPACK_IMPORTED_MODULE_0__.AmbientLight(0x404040);
					scene.add(ambientLight);

					const directionalLight =
						new three__WEBPACK_IMPORTED_MODULE_0__.DirectionalLight(
							0xffffff,
							1
						);
					directionalLight.position.set(5, 5, 5);
					scene.add(directionalLight);

					// Add geometry
					const geometry = new three__WEBPACK_IMPORTED_MODULE_0__.BoxGeometry(
						1,
						1,
						1
					);
					const material =
						new three__WEBPACK_IMPORTED_MODULE_0__.MeshPhongMaterial({
							color: 0x00ff00
						});
					const cube = new three__WEBPACK_IMPORTED_MODULE_0__.Mesh(
						geometry,
						material
					);
					scene.add(cube);

					camera.position.z = 5;

					// Add controls
					const controls =
						new three_examples_jsm_controls_OrbitControls_js__WEBPACK_IMPORTED_MODULE_1__.OrbitControls(
							camera,
							renderer.domElement
						);

					// Add post-processing
					const composer =
						new three_examples_jsm_postprocessing_EffectComposer_js__WEBPACK_IMPORTED_MODULE_5__.EffectComposer(
							renderer
						);
					const renderPass =
						new three_examples_jsm_postprocessing_RenderPass_js__WEBPACK_IMPORTED_MODULE_6__.RenderPass(
							scene,
							camera
						);
					composer.addPass(renderPass);

					const bloomPass =
						new three_examples_jsm_postprocessing_UnrealBloomPass_js__WEBPACK_IMPORTED_MODULE_7__.UnrealBloomPass(
							new three__WEBPACK_IMPORTED_MODULE_0__.Vector2(
								window.innerWidth,
								window.innerHeight
							),
							1.5,
							0.4,
							0.85
						);
					composer.addPass(bloomPass);

					return { scene, camera, renderer, cube, controls, composer };
				}

				// Export loaders for use in other modules
			},
		"./threejs-app/src/materials.js":
			/*!**************************************!*\
  !*** ./threejs-app/src/materials.js ***!
  \**************************************/
			function (
				__unused_webpack___webpack_module__,
				__webpack_exports__,
				__webpack_require__
			) {
				var three__WEBPACK_IMPORTED_MODULE_0___namespace_cache;
				__webpack_require__.d(__webpack_exports__, {
					MaterialFactory: () => MaterialFactory,
					createAdvancedMaterials: () => createAdvancedMaterials
				});
				/* ESM import */ var three__WEBPACK_IMPORTED_MODULE_0__ =
					/* #__PURE__ */ __webpack_require__(
						/*! three */ "webpack/sharing/consume/default/three/three"
					);

				function createAdvancedMaterials() {
					return {
						standard:
							new three__WEBPACK_IMPORTED_MODULE_0__.MeshStandardMaterial({
								color: 0x2194ce,
								roughness: 0.5,
								metalness: 0.5
							}),
						physical:
							new three__WEBPACK_IMPORTED_MODULE_0__.MeshPhysicalMaterial({
								color: 0xff0000,
								roughness: 0.2,
								metalness: 0.8,
								clearcoat: 1.0,
								clearcoatRoughness: 0.1
							}),
						toon: new three__WEBPACK_IMPORTED_MODULE_0__.MeshToonMaterial({
							color: 0x00ff00
						}),
						lambert: new three__WEBPACK_IMPORTED_MODULE_0__.MeshLambertMaterial(
							{
								color: 0x0000ff
							}
						)
					};
				}

				class MaterialFactory {
					constructor() {
						this.cache = new Map();
					}

					getMaterial(type, options) {
						const key = `${type}_${JSON.stringify(options)}`;
						if (!this.cache.has(key)) {
							this.cache.set(
								key,
								new /*#__PURE__*/ (three__WEBPACK_IMPORTED_MODULE_0___namespace_cache ||
									(three__WEBPACK_IMPORTED_MODULE_0___namespace_cache =
										__webpack_require__.t(
											three__WEBPACK_IMPORTED_MODULE_0__,
											2
										)))[type](options)
							);
						}
						return this.cache.get(key);
					}
				}
			}
	};
	/************************************************************************/
	// The module cache
	var __webpack_module_cache__ = {};

	// The require function
	function __webpack_require__(moduleId) {
		// Check if module is in cache
		var cachedModule = __webpack_module_cache__[moduleId];
		if (cachedModule !== undefined) {
			return cachedModule.exports;
		}
		// Create a new module (and put it into the cache)
		var module = (__webpack_module_cache__[moduleId] = {
			exports: {}
		});
		// Execute the module function
		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);

		// Return the exports of the module
		return module.exports;
	}

	// expose the modules object (__webpack_modules__)
	__webpack_require__.m = __webpack_modules__;

	// expose the module cache
	__webpack_require__.c = __webpack_module_cache__;

	/************************************************************************/
	// module_federation/runtime
	(() => {
		if (!__webpack_require__.federation) {
			__webpack_require__.federation = {
				chunkMatcher: function (chunkId) {
					return "webpack_sharing_consume_default_three_three" != chunkId;
				},
				rootOutputDir: ""
			};
		}
	})();
	// webpack/runtime/compat_get_default_export
	(() => {
		// getDefaultExport function for compatibility with non-ESM modules
		__webpack_require__.n = module => {
			var getter =
				module && module.__esModule ? () => module["default"] : () => module;
			__webpack_require__.d(getter, { a: getter });
			return getter;
		};
	})();
	// webpack/runtime/create_fake_namespace_object
	(() => {
		var getProto = Object.getPrototypeOf
			? obj => Object.getPrototypeOf(obj)
			: obj => obj.__proto__;
		var leafPrototypes;
		// create a fake namespace object
		// mode & 1: value is a module id, require it
		// mode & 2: merge all properties of value into the ns
		// mode & 4: return value when already ns object
		// mode & 16: return value when it's Promise-like
		// mode & 8|1: behave like require
		__webpack_require__.t = function (value, mode) {
			if (mode & 1) value = this(value);
			if (mode & 8) return value;
			if (typeof value === "object" && value) {
				if (mode & 4 && value.__esModule) return value;
				if (mode & 16 && typeof value.then === "function") return value;
			}
			var ns = Object.create(null);
			__webpack_require__.r(ns);
			var def = {};
			leafPrototypes = leafPrototypes || [
				null,
				getProto({}),
				getProto([]),
				getProto(getProto)
			];
			for (
				var current = mode & 2 && value;
				typeof current == "object" && !~leafPrototypes.indexOf(current);
				current = getProto(current)
			) {
				Object.getOwnPropertyNames(current).forEach(key => {
					def[key] = () => value[key];
				});
			}
			def["default"] = () => value;
			__webpack_require__.d(ns, def);
			return ns;
		};
	})();
	// webpack/runtime/define_property_getters
	(() => {
		__webpack_require__.d = (exports, definition) => {
			for (var key in definition) {
				if (
					__webpack_require__.o(definition, key) &&
					!__webpack_require__.o(exports, key)
				) {
					Object.defineProperty(exports, key, {
						enumerable: true,
						get: definition[key]
					});
				}
			}
		};
	})();
	// webpack/runtime/ensure_chunk
	(() => {
		__webpack_require__.f = {};
		// This file contains only the entry chunk.
		// The chunk loading function for additional chunks
		__webpack_require__.e = chunkId => {
			return Promise.all(
				Object.keys(__webpack_require__.f).reduce((promises, key) => {
					__webpack_require__.f[key](chunkId, promises);
					return promises;
				}, [])
			);
		};
	})();
	// webpack/runtime/get javascript chunk filename
	(() => {
		// This function allow to reference chunks
		__webpack_require__.u = chunkId => {
			// return url for filenames not based on template

			// return url for filenames based on template
			return "" + chunkId + ".js";
		};
	})();
	// webpack/runtime/global
	(() => {
		__webpack_require__.g = (() => {
			if (typeof globalThis === "object") return globalThis;
			try {
				return this || new Function("return this")();
			} catch (e) {
				if (typeof window === "object") return window;
			}
		})();
	})();
	// webpack/runtime/has_own_property
	(() => {
		__webpack_require__.o = (obj, prop) =>
			Object.prototype.hasOwnProperty.call(obj, prop);
	})();
	// webpack/runtime/load_script
	(() => {
		var inProgress = {};

		var dataWebpackPrefix = "rspack-basic-example:";
		// loadScript function to load a script via script tag
		__webpack_require__.l = function (url, done, key, chunkId) {
			if (inProgress[url]) {
				inProgress[url].push(done);
				return;
			}
			var script, needAttach;
			if (key !== undefined) {
				var scripts = document.getElementsByTagName("script");
				for (var i = 0; i < scripts.length; i++) {
					var s = scripts[i];
					if (
						s.getAttribute("src") == url ||
						s.getAttribute("data-webpack") == dataWebpackPrefix + key
					) {
						script = s;
						break;
					}
				}
			}
			if (!script) {
				needAttach = true;

				script = document.createElement("script");

				script.charset = "utf-8";
				script.timeout = 120;
				if (__webpack_require__.nc) {
					script.setAttribute("nonce", __webpack_require__.nc);
				}
				script.setAttribute("data-webpack", dataWebpackPrefix + key);

				script.src = url;
			}
			inProgress[url] = [done];
			var onScriptComplete = function (prev, event) {
				script.onerror = script.onload = null;
				clearTimeout(timeout);
				var doneFns = inProgress[url];
				delete inProgress[url];
				script.parentNode && script.parentNode.removeChild(script);
				doneFns &&
					doneFns.forEach(function (fn) {
						return fn(event);
					});
				if (prev) return prev(event);
			};
			var timeout = setTimeout(
				onScriptComplete.bind(null, undefined, {
					type: "timeout",
					target: script
				}),
				120000
			);
			script.onerror = onScriptComplete.bind(null, script.onerror);
			script.onload = onScriptComplete.bind(null, script.onload);
			needAttach && document.head.appendChild(script);
		};
	})();
	// webpack/runtime/make_namespace_object
	(() => {
		// define __esModule on exports
		__webpack_require__.r = exports => {
			if (typeof Symbol !== "undefined" && Symbol.toStringTag) {
				Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
			}
			Object.defineProperty(exports, "__esModule", { value: true });
		};
	})();
	// webpack/runtime/rspack_version
	(() => {
		__webpack_require__.rv = () => "1.4.7-alpha.1";
	})();
	// webpack/runtime/sharing
	(() => {
		__webpack_require__.S = {};
		__webpack_require__.initializeSharingData = {
			scopeToSharingDataMapping: {
				default: [
					{
						name: "three-bloom-pass",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_Unreal-fac73e"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/UnrealBloomPass.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/UnrealBloomPass.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-draco-loader",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_DRACOLoader_js"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/DRACOLoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/DRACOLoader.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-effect-composer",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_Effect-e9ec18"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/EffectComposer.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/EffectComposer.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-gltf-loader",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_GLTFLoader_js"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/GLTFLoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/GLTFLoader.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-orbit-controls",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_controls_OrbitControls_js"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/controls/OrbitControls.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/controls/OrbitControls.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-render-pass",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								),
								__webpack_require__.e(
									"node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_RenderPass_js-_c8971"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/RenderPass.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/RenderPass.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three-rgbe-loader",
						version: "0.169.0",
						factory: () =>
							Promise.all([
								__webpack_require__.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_RGBELoader_js"
								),
								__webpack_require__.e(
									"webpack_sharing_consume_default_three_three"
								)
							]).then(
								() => () =>
									__webpack_require__(
										/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/RGBELoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/RGBELoader.js"
									)
							),
						eager: 0,
						singleton: 1
					},
					{
						name: "three",
						version: "0.169.0",
						factory: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_build_three_module_js"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! ../../node_modules/.pnpm/three@0.169.0/node_modules/three/build/three.module.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/build/three.module.js"
										)
								),
						eager: 0,
						singleton: 1,
						requiredVersion: "^0.169.0"
					}
				]
			},
			uniqueName: "rspack-basic-example"
		};
		__webpack_require__.I =
			__webpack_require__.I ||
			function () {
				throw new Error("should have __webpack_require__.I");
			};
	})();
	// webpack/runtime/auto_public_path
	(() => {
		var scriptUrl;

		if (__webpack_require__.g.importScripts)
			scriptUrl = __webpack_require__.g.location + "";
		var document = __webpack_require__.g.document;
		if (!scriptUrl && document) {
			// Technically we could use `document.currentScript instanceof window.HTMLScriptElement`,
			// but an attacker could try to inject `<script>HTMLScriptElement = HTMLImageElement</script>`
			// and use `<img name="currentScript" src="https://attacker.controlled.server/"></img>`
			if (
				document.currentScript &&
				document.currentScript.tagName.toUpperCase() === "SCRIPT"
			)
				scriptUrl = document.currentScript.src;
			if (!scriptUrl) {
				var scripts = document.getElementsByTagName("script");
				if (scripts.length) {
					var i = scripts.length - 1;
					while (i > -1 && (!scriptUrl || !/^http(s?):/.test(scriptUrl)))
						scriptUrl = scripts[i--].src;
				}
			}
		}

		// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration",
		// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.',
		if (!scriptUrl)
			throw new Error("Automatic publicPath is not supported in this browser");
		scriptUrl = scriptUrl
			.replace(/^blob:/, "")
			.replace(/#.*$/, "")
			.replace(/\?.*$/, "")
			.replace(/\/[^\/]+$/, "/");
		__webpack_require__.p = scriptUrl;
	})();
	// webpack/runtime/consumes_loading
	(() => {
		__webpack_require__.consumesLoadingData = {
			chunkMapping: {
				main: [
					"webpack/sharing/consume/default/three-bloom-pass/three/examples/jsm/postprocessing/UnrealBloomPass.js",
					"webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js",
					"webpack/sharing/consume/default/three-render-pass/three/examples/jsm/postprocessing/RenderPass.js",
					"webpack/sharing/consume/default/three-effect-composer/three/examples/jsm/postprocessing/EffectComposer.js",
					"webpack/sharing/consume/default/three-rgbe-loader/three/examples/jsm/loaders/RGBELoader.js",
					"webpack/sharing/consume/default/three-orbit-controls/three/examples/jsm/controls/OrbitControls.js",
					"webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js",
					"webpack/sharing/consume/default/three/three"
				],
				webpack_sharing_consume_default_three_three: [
					"webpack/sharing/consume/default/three/three"
				]
			},
			moduleIdToConsumeDataMapping: {
				"webpack/sharing/consume/default/three-render-pass/three/examples/jsm/postprocessing/RenderPass.js":
					{
						shareScope: "default",
						shareKey: "three-render-pass",
						import: "three/examples/jsm/postprocessing/RenderPass.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_RenderPass_js-_c8970"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/postprocessing/RenderPass.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/RenderPass.js"
										)
								)
					},
				"webpack/sharing/consume/default/three/three": {
					shareScope: "default",
					shareKey: "three",
					import: "three",
					requiredVersion: "^0.169.0",
					strictVersion: false,
					singleton: true,
					eager: false,
					fallback: () =>
						__webpack_require__
							.e(
								"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_build_three_module_js"
							)
							.then(
								() => () =>
									__webpack_require__(
										/*! three */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/build/three.module.js"
									)
							)
				},
				"webpack/sharing/consume/default/three-bloom-pass/three/examples/jsm/postprocessing/UnrealBloomPass.js":
					{
						shareScope: "default",
						shareKey: "three-bloom-pass",
						import: "three/examples/jsm/postprocessing/UnrealBloomPass.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_Unreal-fac73e"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/postprocessing/UnrealBloomPass.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/UnrealBloomPass.js"
										)
								)
					},
				"webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js":
					{
						shareScope: "default",
						shareKey: "three-gltf-loader",
						import: "three/examples/jsm/loaders/GLTFLoader.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_GLTFLoader_js"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/loaders/GLTFLoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/GLTFLoader.js"
										)
								)
					},
				"webpack/sharing/consume/default/three-orbit-controls/three/examples/jsm/controls/OrbitControls.js":
					{
						shareScope: "default",
						shareKey: "three-orbit-controls",
						import: "three/examples/jsm/controls/OrbitControls.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_controls_OrbitControls_js"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/controls/OrbitControls.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/controls/OrbitControls.js"
										)
								)
					},
				"webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js":
					{
						shareScope: "default",
						shareKey: "three-draco-loader",
						import: "three/examples/jsm/loaders/DRACOLoader.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_DRACOLoader_js"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/loaders/DRACOLoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/DRACOLoader.js"
										)
								)
					},
				"webpack/sharing/consume/default/three-rgbe-loader/three/examples/jsm/loaders/RGBELoader.js":
					{
						shareScope: "default",
						shareKey: "three-rgbe-loader",
						import: "three/examples/jsm/loaders/RGBELoader.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_loaders_RGBELoader_js"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/loaders/RGBELoader.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/loaders/RGBELoader.js"
										)
								)
					},
				"webpack/sharing/consume/default/three-effect-composer/three/examples/jsm/postprocessing/EffectComposer.js":
					{
						shareScope: "default",
						shareKey: "three-effect-composer",
						import: "three/examples/jsm/postprocessing/EffectComposer.js",
						requiredVersion: "0.169.0",
						strictVersion: false,
						singleton: true,
						eager: false,
						fallback: () =>
							__webpack_require__
								.e(
									"vendors-node_modules_pnpm_three_0_169_0_node_modules_three_examples_jsm_postprocessing_Effect-e9ec18"
								)
								.then(
									() => () =>
										__webpack_require__(
											/*! three/examples/jsm/postprocessing/EffectComposer.js */ "../../node_modules/.pnpm/three@0.169.0/node_modules/three/examples/jsm/postprocessing/EffectComposer.js"
										)
								)
					}
			},
			initialConsumes: [
				"webpack/sharing/consume/default/three-bloom-pass/three/examples/jsm/postprocessing/UnrealBloomPass.js",
				"webpack/sharing/consume/default/three-draco-loader/three/examples/jsm/loaders/DRACOLoader.js",
				"webpack/sharing/consume/default/three-render-pass/three/examples/jsm/postprocessing/RenderPass.js",
				"webpack/sharing/consume/default/three-effect-composer/three/examples/jsm/postprocessing/EffectComposer.js",
				"webpack/sharing/consume/default/three-rgbe-loader/three/examples/jsm/loaders/RGBELoader.js",
				"webpack/sharing/consume/default/three-orbit-controls/three/examples/jsm/controls/OrbitControls.js",
				"webpack/sharing/consume/default/three-gltf-loader/three/examples/jsm/loaders/GLTFLoader.js",
				"webpack/sharing/consume/default/three/three"
			]
		};
		__webpack_require__.f.consumes =
			__webpack_require__.f.consumes ||
			function () {
				throw new Error("should have __webpack_require__.f.consumes");
			};
	})();
	// webpack/runtime/jsonp_chunk_loading
	(() => {
		// object to store loaded and loading chunks
		// undefined = chunk not loaded, null = chunk preloaded/prefetched
		// [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
		var installedChunks = { main: 0 };

		__webpack_require__.f.j = function (chunkId, promises) {
			// JSONP chunk loading for javascript
			var installedChunkData = __webpack_require__.o(installedChunks, chunkId)
				? installedChunks[chunkId]
				: undefined;
			if (installedChunkData !== 0) {
				// 0 means "already installed".

				// a Promise means "currently loading".
				if (installedChunkData) {
					promises.push(installedChunkData[2]);
				} else {
					if ("webpack_sharing_consume_default_three_three" != chunkId) {
						// setup Promise in chunk cache
						var promise = new Promise(
							(resolve, reject) =>
								(installedChunkData = installedChunks[chunkId] =
									[resolve, reject])
						);
						promises.push((installedChunkData[2] = promise));

						// start chunk loading
						var url = __webpack_require__.p + __webpack_require__.u(chunkId);
						// create error before stack unwound to get useful stacktrace later
						var error = new Error();
						var loadingEnded = function (event) {
							if (__webpack_require__.o(installedChunks, chunkId)) {
								installedChunkData = installedChunks[chunkId];
								if (installedChunkData !== 0)
									installedChunks[chunkId] = undefined;
								if (installedChunkData) {
									var errorType =
										event && (event.type === "load" ? "missing" : event.type);
									var realSrc = event && event.target && event.target.src;
									error.message =
										"Loading chunk " +
										chunkId +
										" failed.\n(" +
										errorType +
										": " +
										realSrc +
										")";
									error.name = "ChunkLoadError";
									error.type = errorType;
									error.request = realSrc;
									installedChunkData[1](error);
								}
							}
						};
						__webpack_require__.l(
							url,
							loadingEnded,
							"chunk-" + chunkId,
							chunkId
						);
					} else installedChunks[chunkId] = 0;
				}
			}
		};
		// install a JSONP callback for chunk loading
		var webpackJsonpCallback = (parentChunkLoadingFunction, data) => {
			var [chunkIds, moreModules, runtime] = data;
			// add "moreModules" to the modules object,
			// then flag all "chunkIds" as loaded and fire callback
			var moduleId,
				chunkId,
				i = 0;
			if (chunkIds.some(id => installedChunks[id] !== 0)) {
				for (moduleId in moreModules) {
					if (__webpack_require__.o(moreModules, moduleId)) {
						__webpack_require__.m[moduleId] = moreModules[moduleId];
					}
				}
				if (runtime) var result = runtime(__webpack_require__);
			}
			if (parentChunkLoadingFunction) parentChunkLoadingFunction(data);
			for (; i < chunkIds.length; i++) {
				chunkId = chunkIds[i];
				if (
					__webpack_require__.o(installedChunks, chunkId) &&
					installedChunks[chunkId]
				) {
					installedChunks[chunkId][0]();
				}
				installedChunks[chunkId] = 0;
			}
		};

		var chunkLoadingGlobal = (self["webpackChunkrspack_basic_example"] =
			self["webpackChunkrspack_basic_example"] || []);
		chunkLoadingGlobal.forEach(webpackJsonpCallback.bind(null, 0));
		chunkLoadingGlobal.push = webpackJsonpCallback.bind(
			null,
			chunkLoadingGlobal.push.bind(chunkLoadingGlobal)
		);
	})();
	// webpack/runtime/rspack_unique_id
	(() => {
		__webpack_require__.ruid = "bundler=rspack@1.4.7-alpha.1";
	})();
	/************************************************************************/
	// module cache are used so entry inlining is disabled
	// startup
	// Load entry module and return exports
	__webpack_require__(
		'@module-federation/runtime/rspack.js!=!data:text/javascript,import __module_federation_bundler_runtime__ from "/Users/bytedance/RustroverProjects/rspack/node_modules/.pnpm/@module-federation+webpack-bundler-runtime@0.16.0/node_modules/@module-federation/webpack-bundler-runtime/dist/index.cjs.cjs";const __module_federation_runtime_plugins__ = [];const __module_federation_remote_infos__ = {};const __module_federation_container_name__ = "threejs_app";const __module_federation_share_strategy__ = "version-first";if((__webpack_require__.initializeSharingData||__webpack_require__.initializeExposesData)&&__webpack_require__.federation){var __webpack_require___remotesLoadingData,__webpack_require___remotesLoadingData1,__webpack_require___initializeSharingData,__webpack_require___consumesLoadingData,__webpack_require___consumesLoadingData1,__webpack_require___initializeExposesData,__webpack_require___consumesLoadingData2;const override=(obj,key,value)=>{if(!obj)return;if(obj[key])obj[key]=value};const merge=(obj,key,fn)=>{const value=fn();if(Array.isArray(value)){var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=[];obj[key].push(...value)}else if(typeof value==="object"&&value!==null){var _obj1,_key1;var _1;(_1=(_obj1=obj)[_key1=key])!==null&&_1!==void 0?_1:_obj1[_key1]={};Object.assign(obj[key],value)}};const early=(obj,key,initial)=>{var _obj,_key;var _;(_=(_obj=obj)[_key=key])!==null&&_!==void 0?_:_obj[_key]=initial()};var __webpack_require___remotesLoadingData_chunkMapping;const remotesLoadingChunkMapping=(__webpack_require___remotesLoadingData_chunkMapping=(__webpack_require___remotesLoadingData=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData===void 0?void 0:__webpack_require___remotesLoadingData.chunkMapping)!==null&&__webpack_require___remotesLoadingData_chunkMapping!==void 0?__webpack_require___remotesLoadingData_chunkMapping:{};var __webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping;const remotesLoadingModuleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping=(__webpack_require___remotesLoadingData1=__webpack_require__.remotesLoadingData)===null||__webpack_require___remotesLoadingData1===void 0?void 0:__webpack_require___remotesLoadingData1.moduleIdToRemoteDataMapping)!==null&&__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping!==void 0?__webpack_require___remotesLoadingData_moduleIdToRemoteDataMapping:{};var __webpack_require___initializeSharingData_scopeToSharingDataMapping;const initializeSharingScopeToInitDataMapping=(__webpack_require___initializeSharingData_scopeToSharingDataMapping=(__webpack_require___initializeSharingData=__webpack_require__.initializeSharingData)===null||__webpack_require___initializeSharingData===void 0?void 0:__webpack_require___initializeSharingData.scopeToSharingDataMapping)!==null&&__webpack_require___initializeSharingData_scopeToSharingDataMapping!==void 0?__webpack_require___initializeSharingData_scopeToSharingDataMapping:{};var __webpack_require___consumesLoadingData_chunkMapping;const consumesLoadingChunkMapping=(__webpack_require___consumesLoadingData_chunkMapping=(__webpack_require___consumesLoadingData=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData===void 0?void 0:__webpack_require___consumesLoadingData.chunkMapping)!==null&&__webpack_require___consumesLoadingData_chunkMapping!==void 0?__webpack_require___consumesLoadingData_chunkMapping:{};var __webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping;const consumesLoadingModuleToConsumeDataMapping=(__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping=(__webpack_require___consumesLoadingData1=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData1===void 0?void 0:__webpack_require___consumesLoadingData1.moduleIdToConsumeDataMapping)!==null&&__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping!==void 0?__webpack_require___consumesLoadingData_moduleIdToConsumeDataMapping:{};const consumesLoadinginstalledModules={};const initializeSharingInitPromises=[];const initializeSharingInitTokens={};const containerShareScope=(__webpack_require___initializeExposesData=__webpack_require__.initializeExposesData)===null||__webpack_require___initializeExposesData===void 0?void 0:__webpack_require___initializeExposesData.shareScope;for(const key in __module_federation_bundler_runtime__){__webpack_require__.federation[key]=__module_federation_bundler_runtime__[key]}early(__webpack_require__.federation,"consumesLoadingModuleToHandlerMapping",()=>{const consumesLoadingModuleToHandlerMapping={};for(let[moduleId,data]of Object.entries(consumesLoadingModuleToConsumeDataMapping)){consumesLoadingModuleToHandlerMapping[moduleId]={getter:data.fallback,shareInfo:{shareConfig:{fixedDependencies:false,requiredVersion:data.requiredVersion,strictVersion:data.strictVersion,singleton:data.singleton,eager:data.eager},scope:[data.shareScope]},shareKey:data.shareKey}}return consumesLoadingModuleToHandlerMapping});early(__webpack_require__.federation,"initOptions",()=>({}));early(__webpack_require__.federation.initOptions,"name",()=>__module_federation_container_name__);early(__webpack_require__.federation.initOptions,"shareStrategy",()=>__module_federation_share_strategy__);early(__webpack_require__.federation.initOptions,"shared",()=>{const shared={};for(let[scope,stages]of Object.entries(initializeSharingScopeToInitDataMapping)){for(let stage of stages){if(typeof stage==="object"&&stage!==null){const{name,version,factory,eager,singleton,requiredVersion,strictVersion}=stage;const shareConfig={};const isValidValue=function(val){return typeof val!=="undefined"};if(isValidValue(singleton)){shareConfig.singleton=singleton}if(isValidValue(requiredVersion)){shareConfig.requiredVersion=requiredVersion}if(isValidValue(eager)){shareConfig.eager=eager}if(isValidValue(strictVersion)){shareConfig.strictVersion=strictVersion}const options={version,scope:[scope],shareConfig,get:factory};if(shared[name]){shared[name].push(options)}else{shared[name]=[options]}}}}return shared});merge(__webpack_require__.federation.initOptions,"remotes",()=>Object.values(__module_federation_remote_infos__).flat().filter(remote=>remote.externalType==="script"));merge(__webpack_require__.federation.initOptions,"plugins",()=>__module_federation_runtime_plugins__);early(__webpack_require__.federation,"bundlerRuntimeOptions",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions,"remotes",()=>({}));early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"chunkMapping",()=>remotesLoadingChunkMapping);early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToExternalAndNameMapping",()=>{const remotesLoadingIdToExternalAndNameMappingMapping={};for(let[moduleId,data]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){remotesLoadingIdToExternalAndNameMappingMapping[moduleId]=[data.shareScope,data.name,data.externalModuleId,data.remoteName]}return remotesLoadingIdToExternalAndNameMappingMapping});early(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"webpackRequire",()=>__webpack_require__);merge(__webpack_require__.federation.bundlerRuntimeOptions.remotes,"idToRemoteMap",()=>{const idToRemoteMap={};for(let[id,remoteData]of Object.entries(remotesLoadingModuleIdToRemoteDataMapping)){const info=__module_federation_remote_infos__[remoteData.remoteName];if(info)idToRemoteMap[id]=info}return idToRemoteMap});override(__webpack_require__,"S",__webpack_require__.federation.bundlerRuntime.S);if(__webpack_require__.federation.attachShareScopeMap){__webpack_require__.federation.attachShareScopeMap(__webpack_require__)}override(__webpack_require__.f,"remotes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.remotes({chunkId,promises,chunkMapping:remotesLoadingChunkMapping,idToExternalAndNameMapping:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToExternalAndNameMapping,idToRemoteMap:__webpack_require__.federation.bundlerRuntimeOptions.remotes.idToRemoteMap,webpackRequire:__webpack_require__}));override(__webpack_require__.f,"consumes",(chunkId,promises)=>__webpack_require__.federation.bundlerRuntime.consumes({chunkId,promises,chunkMapping:consumesLoadingChunkMapping,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping,installedModules:consumesLoadinginstalledModules,webpackRequire:__webpack_require__}));override(__webpack_require__,"I",(name,initScope)=>__webpack_require__.federation.bundlerRuntime.I({shareScopeName:name,initScope,initPromises:initializeSharingInitPromises,initTokens:initializeSharingInitTokens,webpackRequire:__webpack_require__}));override(__webpack_require__,"initContainer",(shareScope,initScope,remoteEntryInitOptions)=>__webpack_require__.federation.bundlerRuntime.initContainerEntry({shareScope,initScope,remoteEntryInitOptions,shareScopeKey:containerShareScope,webpackRequire:__webpack_require__}));override(__webpack_require__,"getContainer",(module1,getScope)=>{var moduleMap=__webpack_require__.initializeExposesData.moduleMap;__webpack_require__.R=getScope;getScope=Object.prototype.hasOwnProperty.call(moduleMap,module1)?moduleMap[module1]():Promise.resolve().then(()=>{throw new Error(\'Module "\'+module1+\'" does not exist in container.\')});__webpack_require__.R=undefined;return getScope});__webpack_require__.federation.instance=__webpack_require__.federation.runtime.init(__webpack_require__.federation.initOptions);if((__webpack_require___consumesLoadingData2=__webpack_require__.consumesLoadingData)===null||__webpack_require___consumesLoadingData2===void 0?void 0:__webpack_require___consumesLoadingData2.initialConsumes){__webpack_require__.federation.bundlerRuntime.installInitialConsumes({webpackRequire:__webpack_require__,installedModules:consumesLoadinginstalledModules,initialConsumes:__webpack_require__.consumesLoadingData.initialConsumes,moduleToHandlerMapping:__webpack_require__.federation.consumesLoadingModuleToHandlerMapping})}}'
	);
	var __webpack_exports__ = __webpack_require__("./threejs-app/index.js");
})();
