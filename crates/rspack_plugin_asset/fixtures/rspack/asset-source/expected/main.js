(function() {// var __webpack_modules__ = ({});
// replace here to modules
var __webpack_modules__ = {
"./data.txt": function (module, exports, __webpack_require__) {
"use strict";
module.exports = "- Isn't Rspack a gamechanging bundler?\n  - Hella yeah!";},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _dataTxt = __webpack_require__.interopRequire(__webpack_require__("./data.txt"));
console.log(_dataTxt.default);
},

};
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
		// no module.id needed
		// no module.loaded needed
		exports: {}
	});

	// Execute the module function
	var execOptions = {
		id: moduleId,
		module: module,
		factory: __webpack_modules__[moduleId],
		require: __webpack_require__
	};
	__webpack_require__.i.forEach(function (handler) {
		handler(execOptions);
	});
	module = execOptions.module;
	execOptions.factory.call(
		module.exports,
		module,
		module.exports,
		execOptions.require
	);

	// Return the exports of the module
	return module.exports;
}

// expose the modules object (__webpack_modules__)
__webpack_require__.m = __webpack_modules__;
// expose the module cache
__webpack_require__.c = __webpack_module_cache__;
// expose the module execution interceptor
__webpack_require__.i = [];
(function(){
__webpack_require__.chunkId = 'main'})();
            (function(){
              __webpack_require__.hu = function (chunkId) {
                return '' + chunkId + '.hot-update.js';
              }
            })();// hot runtime
(function () {
	var currentModuleData = {};
	var installedModules = __webpack_require__.c;

	// module and require creation
	var currentChildModule;
	var currentParents = [];

	// status
	var registeredStatusHandlers = [];
	var currentStatus = "idle";

	// while downloading
	// TODO: not needed in rspack temporary,
	// TODO: because we transfer all changed modules.
	var blockingPromises = 0;
	var blockingPromisesWaiting = [];

	// The update info
	var currentUpdateApplyHandlers;
	var queuedInvalidatedModules;

	__webpack_require__.hmrD = currentModuleData;
	__webpack_require__.i.push(function (options) {
		var module = options.module;
		var require = createRequire(options.require, options.id);
		module.hot = createModuleHotObject(options.id, module);
		module.parents = currentParents;
		module.children = [];
		currentParents = [];
		options.require = require;
	});

	__webpack_require__.hmrC = {};
	__webpack_require__.hmrI = {};

	function createRequire(require, moduleId) {
		var me = installedModules[moduleId];
		if (!me) {
			return require;
		}
		var fn = function (request) {
			if (me.hot.active) {
				if (installedModules[request]) {
					var parents = installedModules[request].parents;
					if (parents.indexOf(moduleId) === -1) {
						parents.push(moduleId);
					}
				} else {
					currentParents = [moduleId];
					currentChildModule = request;
				}
				if (me.children.indexOf(request) === -1) {
					me.children.push(request);
				}
			} else {
				console.log(
					"[HMR] unexpected require(" +
						request +
						") from disposed module " +
						moduleId
				);
				currentParents = [];
			}
			return require(request);
		};
		var createPropertyDescriptor = function (name) {
			return {
				configurable: true,
				enumerable: true,
				get: function () {
					return require[name];
				},
				set: function (value) {
					require[name] = value;
				}
			};
		};
		for (var name in require) {
			if (Object.prototype.hasOwnProperty.call(require, name) && name !== "e") {
				Object.defineProperty(fn, name, createPropertyDescriptor(name));
			}
		}

		fn.e = function (chunkId) {
			return trackBlockingPromise(require.e(chunkId));
		};

		return fn;
	}

	function createModuleHotObject(moduleId, me) {
		var _main = currentChildModule !== moduleId;
		var hot = {
			_acceptedDependencies: {},
			_acceptedErrorHandlers: {},
			_declinedDependencies: {},
			_selfAccepted: false,
			_selfDeclined: false,
			_selfInvalidated: false,
			_disposeHandlers: [],
			_main: _main,
			_requireSelf: function () {
				currentParents = me.parents.slice();
				currentChildModule = _main ? undefined : moduleId;
				__webpack_require__(moduleId);
			},
			active: true,
			accept: function (dep, callback, errorHandler) {
				if (dep === undefined) {
					hot._selfAccepted = true;
				} else if (typeof dep === "function") {
					hot._selfAccepted = dep;
				} else if (typeof dep === "object" && dep !== null) {
					for (var i = 0; i < dep.length; i++) {
						hot._acceptedDependencies[dep[i]] = callback || function () {};
						hot._acceptedErrorHandlers[dep[i]] = errorHandler;
					}
				} else {
					hot._acceptedDependencies[dep] = callback || function () {};
					hot._acceptedErrorHandlers[dep] = errorHandler;
				}
			},
			decline: function (dep) {
				if (dep === undefined) {
					hot._selfDeclined = true;
				} else if (typeof dep === "object" && dep !== null) {
					for (var i = 0; i < dep.length; i++) {
						hot._declinedDependencies[dep[i]] = true;
					}
				} else {
					hot._declinedDependencies[dep] = true;
				}
			},
			dispose: function (callback) {
				hot._disposeHandlers.push(callback);
			},
			addDisposeHandler: function (callback) {
				hot._disposeHandlers.push(callback);
			},
			removeDisposeHandler: function (callback) {
				var idx = hot._disposeHandlers.indexOf(callback);
				if (idx > 0) {
					hot._disposeHandlers.splice(idx, 1);
				}
			},
			invalidate: function () {
				this._selfInvalidated = true;
				switch (currentStatus) {
					case "idle":
						currentUpdateApplyHandlers = [];
						Object.keys(__webpack_require__.hmrI).forEach(function (key) {
							__webpack_require__.hmrI[key](
								moduleId,
								currentUpdateApplyHandlers
							);
						});
						setStatus("ready");
						break;
					case "ready":
						Object.keys(__webpack_require__.hmrI).forEach(function (key) {
							__webpack_require__.hmrI[key](
								moduleId,
								currentUpdateApplyHandlers
							);
						});
						break;
					case "prepare":
					case "check":
					case "dispose":
					case "apply":
						(queuedInvalidatedModules = queuedInvalidatedModules || []).push(
							moduleId
						);
						break;
					default:
						break;
				}
			},
			check: hotCheck,
			apply: hotApply,
			status: function (l) {
				if (!l) {
					return currentStatus;
				}
				registeredStatusHandlers.push(l);
			},
			addStatusHandler: function (l) {
				registeredStatusHandlers.push(l);
			},
			removeStatusHandler: function (l) {
				var idx = registeredStatusHandlers.indexOf(l);
				if (idx >= 0) registeredStatusHandlers.splice(idx, 1);
			},
			data: currentModuleData[moduleId]
		};
		currentChildModule = undefined;
		return hot;
	}

	function setStatus(newStats) {
		currentStatus = newStats;
		var results = [];
		for (var i = 0; i < registeredStatusHandlers.length; i++) {
			results[i] = registeredStatusHandlers[i].call(null, newStats);
		}
		return Promise.all(results);
	}

	function unblock() {
		if (--blockingPromises === 0) {
			setStatus("ready").then(function () {
				if (blockingPromises === 0) {
					var list = blockingPromisesWaiting;
					blockingPromisesWaiting = [];
					for (var i = 0; i < list.length; i++) {
						list[i]();
					}
				}
			});
		}
	}

	function trackBlockingPromise(promise) {
		switch (currentStatus) {
			case "ready":
				setStatus("prepare");
			case "prepare":
				blockingPromises++;
				promise.then(unblock, unblock);
				return promise;
			default:
				return promise;
		}
	}

	function waitForBlockingPromises(fn) {
		if (blockingPromises === 0) {
			return fn();
		}
		return new Promise(function (resolve) {
			blockingPromisesWaiting.push(function () {
				resolve(fn());
			});
		});
	}

	function hotCheck(applyOnUpdate) {
		if (currentStatus !== "idle") {
			throw new Error("check() is only allowed in idle status");
		}
		return setStatus("check")
			.then(__webpack_require__.hmrM)
			.then(function (update) {
				if (!update) {
					return setStatus(applyInvalidatedModules() ? "ready" : "idle").then(
						function () {
							return null;
						}
					);
				}

				return setStatus("prepare").then(function () {
					var updatedModules = [];
					currentUpdateApplyHandlers = [];

					return Promise.all(
						Object.keys(__webpack_require__.hmrC).reduce(function (
							promises,
							key
						) {
							__webpack_require__.hmrC[key](
								update.c,
								update.r,
								update.m,
								promises,
								currentUpdateApplyHandlers,
								updatedModules
							);
							return promises;
						},
						[])
					).then(function () {
						return waitForBlockingPromises(function () {
							if (applyOnUpdate) {
								return internalApply(applyOnUpdate);
							} else {
								return setStatus("ready").then(function () {
									return updatedModules;
								});
							}
						});
					});
				});
			});
	}

	function hotApply(options) {
		if (currentStatus !== "ready") {
			return Promise.resolve().then(function () {
				throw Error(
					"apply() is only allowed in ready status (state: " +
						currentStatus +
						")"
				);
			});
		}
		return internalApply(options);
	}

	function internalApply(options) {
		options = options || {};
		applyInvalidatedModules();
		var results = currentUpdateApplyHandlers.map(function (handler) {
			return handler(options);
		});
		currentUpdateApplyHandlers = undefined;
		var errors = results
			.map(function (r) {
				return r.error;
			})
			.filter(Boolean);

		if (errors.length > 0) {
			return setStatus("abort").then(function () {
				throw errors[0];
			});
		}

		var disposePromise = setStatus("dispose");

		results.forEach(function (result) {
			if (result.dispose) {
				result.dispose();
			}
		});

		var applyPromise = setStatus("apply");

		var error;
		var reportError = function (err) {
			if (!error) {
				error = err;
			}
		};

		var outdatedModules = [];
		results.forEach(function (result) {
			if (result.apply) {
				var modules = result.apply(reportError);
				if (modules) {
					for (var i = 0; i < modules.length; i++) {
						outdatedModules.push(modules[i]);
					}
				}
			}
		});

		return Promise.all([disposePromise, applyPromise]).then(function () {
			if (error) {
				return setStatus("fail").then(function () {
					throw error;
				});
			}

			if (queuedInvalidatedModules) {
				return internalApply(options).then(function (list) {
					outdatedModules.forEach(function (moduleId) {
						if (list.indexOf(moduleId) < 0) {
							list.push(moduleId);
						}
					});
					return list;
				});
			}

			return setStatus("idle").then(function () {
				return outdatedModules;
			});
		});
	}

	function applyInvalidatedModules() {
		if (queuedInvalidatedModules) {
			if (!currentUpdateApplyHandlers) {
				currentUpdateApplyHandlers = [];
			}
			Object.keys(__webpack_require__.hmrI).forEach(function (key) {
				queuedInvalidatedModules.forEach(function (moduleId) {
					__webpack_require__.hmrI[key](moduleId, currentUpdateApplyHandlers);
				});
			});
			queuedInvalidatedModules = undefined;
			return true;
		}
	}
})();
(function () {
	var installedChunks = (__webpack_require__.hmrS_jsonp =
		__webpack_require__.hmrS_jsonp || {
			[__webpack_require__.chunkId]: 0
		});

	var currentUpdatedModulesList;
	var waitingUpdateResolves = {};
	function loadUpdateChunk(chunkId, updatedModulesList) {
		currentUpdatedModulesList = updatedModulesList;
		return new Promise((resolve, reject) => {
			// start update chunk loading
			var url = __webpack_require__.p + __webpack_require__.hu(chunkId);

			waitingUpdateResolves[chunkId] = resolve;
			// create error before stack unwound to get useful stacktrace later
			var error = new Error();
			var loadingEnded = event => {
				if (waitingUpdateResolves[chunkId]) {
					waitingUpdateResolves[chunkId] = undefined;
					var errorType =
						event && (event.type === "load" ? "missing" : event.type);
					var realSrc = event && event.target && event.target.src;
					error.message =
						"Loading hot update chunk " +
						chunkId +
						" failed.\n(" +
						errorType +
						": " +
						realSrc +
						")";
					error.name = "ChunkLoadError";
					error.type = errorType;
					error.request = realSrc;
					reject(error);
				}
			};
			__webpack_require__.l(url, loadingEnded);
		});
	}

	self["hotUpdate"] = (chunkId, moreModules, runtime) => {
		for (var moduleId in moreModules) {
			if (__webpack_require__.o(moreModules, moduleId)) {
				currentUpdate[moduleId] = moreModules[moduleId];
				if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId);
			}
		}
		if (runtime) currentUpdateRuntime.push(runtime);
		if (waitingUpdateResolves[chunkId]) {
			waitingUpdateResolves[chunkId]();
			waitingUpdateResolves[chunkId] = undefined;
		}
	};

	__webpack_require__.hmrM = function () {
		if (typeof fetch === "undefined")
			throw new Error("No browser support: need fetch API");
		// TODO: should use `hmrF()`
		var f = __webpack_require__.chunkId + ".hot-update.json";
		return fetch(__webpack_require__.p + f).then(response => {
			if (response.status === 404) return; // no update available
			if (!response.ok)
				throw new Error(
					"Failed to fetch update manifest " + response.statusText
				);
			return response.json();
		});
	};

	var currentUpdateChunks;
	var currentUpdate;
	var currentUpdateRemovedChunks;
	var currentUpdateRuntime;
	function applyHandler(options) {
		currentUpdateChunks = undefined;
		function getAffectedModuleEffects(updateModuleId) {
			var outdatedModules = [updateModuleId];
			var outdatedDependencies = {};
			var queue = outdatedModules.map(function (id) {
				return {
					chain: [id],
					id: id
				};
			});
			while (queue.length > 0) {
				var queueItem = queue.pop();
				var moduleId = queueItem.id;
				var chain = queueItem.chain;
				var module = __webpack_require__.c[moduleId];
				if (
					!module ||
					(module.hot._selfAccepted && !module.hot._selfInvalidated)
				) {
					continue;
				}

				if (module.hot._selfDeclined) {
					return {
						type: "self-declined",
						chain: chain,
						moduleId: moduleId
					};
				}

				if (module.hot._main) {
					return {
						type: "unaccepted",
						chain: chain,
						moduleId: moduleId
					};
				}

				for (var i = 0; i < module.parents.length; i++) {
					var parentId = module.parents[i];
					var parent = __webpack_require__.c[parentId];
					if (!parent) {
						continue;
					}
					if (parent.hot._declinedDependencies[moduleId]) {
						return {
							type: "declined",
							chain: chain.concat([parentId]),
							moduleId: moduleId,
							parentId: parentId
						};
					}
					if (outdatedModules.indexOf(parentId) !== -1) {
						continue;
					}
					if (parent.hot._acceptedDependencies[moduleId]) {
						if (!outdatedDependencies[parentId]) {
							outdatedDependencies[parentId] = [];
						}
						addAllToSet(outdatedDependencies[parentId], [moduleId]);
						continue;
					}
					delete outdatedDependencies[parentId];
					outdatedModules.push(parentId);
					queue.push({
						chain: chain.concat([parentId]),
						id: parentId
					});
				}
			}

			return {
				type: "accepted",
				moduleId: updateModuleId,
				outdatedModules: outdatedModules,
				outdatedDependencies: outdatedDependencies
			};
		}

		function addAllToSet(a, b) {
			for (var i = 0; i < b.length; i++) {
				var item = b[i];
				if (a.indexOf(item) === -1) a.push(item);
			}
		}

		var outdatedDependencies = {};
		var outdatedModules = [];
		var appliedUpdate = {};

		var warnUnexpectedRequire = function warnUnexpectedRequire(module) {
			console.warn(
				"[HMR] unexpected require(" + module.id + ") to disposed module"
			);
		};

		for (var moduleId in currentUpdate) {
			if (__webpack_require__.o(currentUpdate, moduleId)) {
				var newModuleFactory = currentUpdate[moduleId];
				var result;
				if (newModuleFactory) {
					result = getAffectedModuleEffects(moduleId);
				} else {
					result = {
						type: "disposed",
						moduleId: moduleId
					};
				}
				var abortError = false;
				var doApply = false;
				var doDispose = false;
				var chainInfo = "";
				if (result.chain) {
					chainInfo = "\nUpdate propagation: " + result.chain.join(" -> ");
				}
				switch (result.type) {
					case "self-declined":
						if (options.onDeclined) options.onDeclined(result);
						if (!options.ignoreDeclined)
							abortError = new Error(
								"Aborted because of self decline: " +
									result.moduleId +
									chainInfo
							);
						break;
					case "declined":
						if (options.onDeclined) options.onDeclined(result);
						if (!options.ignoreDeclined)
							abortError = new Error(
								"Aborted because of declined dependency: " +
									result.moduleId +
									" in " +
									result.parentId +
									chainInfo
							);
						break;
					case "unaccepted":
						if (options.onUnaccepted) options.onUnaccepted(result);
						if (!options.ignoreUnaccepted)
							abortError = new Error(
								"Aborted because " + moduleId + " is not accepted" + chainInfo
							);
						break;
					case "accepted":
						if (options.onAccepted) options.onAccepted(result);
						doApply = true;
						break;
					case "disposed":
						if (options.onDisposed) options.onDisposed(result);
						doDispose = true;
						break;
					default:
						throw new Error("Unexception type " + result.type);
				}
				if (abortError) {
					return {
						error: abortError
					};
				}
				if (doApply) {
					appliedUpdate[moduleId] = newModuleFactory;
					addAllToSet(outdatedModules, result.outdatedModules);
					for (moduleId in result.outdatedDependencies) {
						if (__webpack_require__.o(result.outdatedDependencies, moduleId)) {
							if (!outdatedDependencies[moduleId])
								outdatedDependencies[moduleId] = [];
							addAllToSet(
								outdatedDependencies[moduleId],
								result.outdatedDependencies[moduleId]
							);
						}
					}
				}
				if (doDispose) {
					addAllToSet(outdatedModules, [result.moduleId]);
					appliedUpdate[moduleId] = warnUnexpectedRequire;
				}
			}
		}
		currentUpdate = undefined;

		var outdatedSelfAcceptedModules = [];
		for (var j = 0; j < outdatedModules.length; j++) {
			var outdatedModuleId = outdatedModules[j];
			var module = __webpack_require__.c[outdatedModuleId];
			if (
				module &&
				(module.hot._selfAccepted || module.hot._main) &&
				// removed self-accepted modules should not be required
				appliedUpdate[outdatedModuleId] !== warnUnexpectedRequire &&
				// when called invalidate self-accepting is not possible
				!module.hot._selfInvalidated
			) {
				outdatedSelfAcceptedModules.push({
					module: outdatedModuleId,
					require: module.hot._requireSelf,
					errorHandler: module.hot._selfAccepted
				});
			}
		}

		var moduleOutdatedDependencies;
		return {
			dispose: function () {
				currentUpdateRemovedChunks.forEach(function (chunkId) {
					delete installedChunks[chunkId];
				});
				currentUpdateRemovedChunks = undefined;

				var idx;
				var queue = outdatedModules.slice();
				while (queue.length > 0) {
					var moduleId = queue.pop();
					var module = __webpack_require__.c[moduleId];
					if (!module) continue;

					var data = {};

					// Call dispose handlers
					var disposeHandlers = module.hot._disposeHandlers;
					for (j = 0; j < disposeHandlers.length; j++) {
						disposeHandlers[j].call(null, data);
					}
					__webpack_require__.hmrD[moduleId] = data;

					module.hot.active = false;

					delete __webpack_require__.c[moduleId];

					delete outdatedDependencies[moduleId];

					for (j = 0; j < module.children.length; j++) {
						var child = __webpack_require__.c[module.children[j]];
						if (!child) continue;
						idx = child.parents.indexOf(moduleId);
						if (idx >= 0) {
							child.parents.splice(idx, 1);
						}
					}
				}

				var dependency;
				for (var outdatedModuleId in outdatedDependencies) {
					if (__webpack_require__.o(outdatedDependencies, outdatedModuleId)) {
						module = __webpack_require__.c[outdatedModuleId];
						if (module) {
							moduleOutdatedDependencies =
								outdatedDependencies[outdatedModuleId];
							for (j = 0; j < moduleOutdatedDependencies.length; j++) {
								dependency = moduleOutdatedDependencies[j];
								idx = module.children.indexOf(dependency);
								if (idx >= 0) module.children.splice(idx, 1);
							}
						}
					}
				}
			},
			apply: function (reportError) {
				// insert new code
				for (var updateModuleId in appliedUpdate) {
					if (__webpack_require__.o(appliedUpdate, updateModuleId)) {
						__webpack_require__.m[updateModuleId] =
							appliedUpdate[updateModuleId];
					}
				}

				// run new runtime modules
				for (var i = 0; i < currentUpdateRuntime.length; i++) {
					currentUpdateRuntime[i](__webpack_require__);
				}

				// call accept handlers
				for (var outdatedModuleId in outdatedDependencies) {
					if (__webpack_require__.o(outdatedDependencies, outdatedModuleId)) {
						var module = __webpack_require__.c[outdatedModuleId];
						if (module) {
							moduleOutdatedDependencies =
								outdatedDependencies[outdatedModuleId];
							var callbacks = [];
							var errorHandlers = [];
							var dependenciesForCallbacks = [];
							for (var j = 0; j < moduleOutdatedDependencies.length; j++) {
								var dependency = moduleOutdatedDependencies[j];
								var acceptCallback =
									module.hot._acceptedDependencies[dependency];
								var errorHandler =
									module.hot._acceptedErrorHandlers[dependency];
								if (acceptCallback) {
									if (callbacks.indexOf(acceptCallback) !== -1) continue;
									callbacks.push(acceptCallback);
									errorHandlers.push(errorHandler);
									dependenciesForCallbacks.push(dependency);
								}
							}
							for (var k = 0; k < callbacks.length; k++) {
								try {
									callbacks[k].call(null, moduleOutdatedDependencies);
								} catch (err) {
									if (typeof errorHandlers[k] === "function") {
										try {
											errorHandlers[k](err, {
												moduleId: outdatedModuleId,
												dependencyId: dependenciesForCallbacks[k]
											});
										} catch (err2) {
											if (options.onErrored) {
												options.onErrored({
													type: "accept-error-handler-errored",
													moduleId: outdatedModuleId,
													dependencyId: dependenciesForCallbacks[k],
													error: err2,
													originalError: err
												});
											}
											if (!options.ignoreErrored) {
												reportError(err2);
												reportError(err);
											}
										}
									} else {
										if (options.onErrored) {
											options.onErrored({
												type: "accept-errored",
												moduleId: outdatedModuleId,
												dependencyId: dependenciesForCallbacks[k],
												error: err
											});
										}
										if (!options.ignoreErrored) {
											reportError(err);
										}
									}
								}
							}
						}
					}
				}

				// Load self accepted modules
				for (var o = 0; o < outdatedSelfAcceptedModules.length; o++) {
					var item = outdatedSelfAcceptedModules[o];
					var moduleId = item.module;
					try {
						item.require(moduleId);
					} catch (err) {
						if (typeof item.errorHandler === "function") {
							try {
								item.errorHandler(err, {
									moduleId: moduleId,
									module: __webpack_require__.c[moduleId]
								});
							} catch (err2) {
								if (options.onErrored) {
									options.onErrored({
										type: "self-accept-error-handler-errored",
										moduleId: moduleId,
										error: err2,
										originalError: err
									});
								}
								if (!options.ignoreErrored) {
									reportError(err2);
									reportError(err);
								}
							}
						} else {
							if (options.onErrored) {
								options.onErrored({
									type: "self-accept-errored",
									moduleId: moduleId,
									error: err
								});
							}
							if (!options.ignoreErrored) {
								reportError(err);
							}
						}
					}
				}

				return outdatedModules;
			}
		};
	}

	__webpack_require__.hmrI.jsonp = function (moduleId, applyHandlers) {
		if (!currentUpdate) {
			currentUpdate = {};
			currentUpdateRuntime = [];
			currentUpdateRemovedChunks = [];
			applyHandlers.push(applyHandler);
		}
		if (!__webpack_require__.o(currentUpdate, moduleId)) {
			currentUpdate[moduleId] = __webpack_require__.m[moduleId];
		}
	};

	// TODO: fetch is not needed
	__webpack_require__.hmrC.jsonp = function (
		chunkIds,
		removedChunks,
		removedModules,
		promises,
		applyHandlers,
		updatedModulesList
	) {
		applyHandlers.push(applyHandler);
		currentUpdateChunks = {};
		currentUpdateRemovedChunks = removedChunks;
		currentUpdate = removedModules.reduce(function (obj, key) {
			obj[key] = false;
			return obj;
		}, {});
		currentUpdateRuntime = [];
		chunkIds.forEach(function (chunkId) {
			if (
				__webpack_require__.o(installedChunks, chunkId) &&
				installedChunks[chunkId] !== undefined
			) {
				promises.push(loadUpdateChunk(chunkId, updatedModulesList));
				currentUpdateChunks[chunkId] = true;
			} else {
				currentUpdateChunks[chunkId] = false;
			}
		});
		// TODO:
		// if (__webpack_require.f) {}
	};
})();
(function () {
	function _getRequireCache(nodeInterop) {
		if (typeof WeakMap !== "function") return null;

		var cacheBabelInterop = new WeakMap();
		var cacheNodeInterop = new WeakMap();
		return (_getRequireCache = function (nodeInterop) {
			return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
		})(nodeInterop);
	}

	__webpack_require__.interopRequire = function (obj, nodeInterop) {
		if (!nodeInterop && obj && obj.__esModule) {
			return obj;
		}

		if (
			obj === null ||
			(typeof obj !== "object" && typeof obj !== "function")
		) {
			return { default: obj };
		}

		var cache = _getRequireCache(nodeInterop);
		if (cache && cache.has(obj)) {
			return cache.get(obj);
		}

		var newObj = {};
		var hasPropertyDescriptor =
			Object.defineProperty && Object.getOwnPropertyDescriptor;
		for (var key in obj) {
			if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
				var desc = hasPropertyDescriptor
					? Object.getOwnPropertyDescriptor(obj, key)
					: null;
				if (desc && (desc.get || desc.set)) {
					Object.defineProperty(newObj, key, desc);
				} else {
					newObj[key] = obj[key];
				}
			}
		}
		newObj.default = obj;
		if (cache) {
			cache.set(obj, newObj);
		}
		return newObj;
	};
})();
(function () {
	var inProgress = {};
	var dataWebpackPrefix = "webpack:";
	// loadScript function to load a script via script tag
	__webpack_require__.l = function loadScript(url, done, key, chunkId) {
		// TODO add this after hash
		// if (inProgress[url]) {
		// 	inProgress[url].push(done);
		// 	return;
		// }
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
(function () {
	__webpack_require__.o = function (obj, prop) {
		return Object.prototype.hasOwnProperty.call(obj, prop);
	};
})();
(function () {
	__webpack_require__.p = "/";
})();
__webpack_require__("./index.js");})()