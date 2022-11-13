(function () {
	var installedChunks = (runtime.__rspack_require__.hmrS_jsonp = runtime
		.__rspack_require__.hmrS_jsonp || {
		[runtime.__rspack_require__.chunkId]: 0
	});

	var currentUpdatedModulesList;
	var waitingUpdateResolves = {};
	function loadUpdateChunk(chunkId, updatedModulesList) {
		currentUpdatedModulesList = updatedModulesList;
		return new Promise((resolve, reject) => {
			// start update chunk loading
			var url =
				runtime.__rspack_require__.p + runtime.__rspack_require__.hu(chunkId);

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
			runtime.__rspack_require__.l(url, loadingEnded);
		});
	}

	self["hotUpdate"] = (chunkId, moreModules, runtime) => {
		for (var moduleId in moreModules) {
			if (
				self["__rspack_runtime__"].__rspack_require__.o(moreModules, moduleId)
			) {
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

	runtime.__rspack_require__.hmrM = function () {
		if (typeof fetch === "undefined")
			throw new Error("No browser support: need fetch API");
		// TODO: should use `hmrF()`
		var f = runtime.__rspack_require__.chunkId + ".hot-update.json";
		return fetch(runtime.__rspack_require__.p + f).then(response => {
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
				var module = runtime.moduleCache[moduleId];
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
					var parent = runtime.moduleCache[parentId];
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
			if (runtime.__rspack_require__.o(currentUpdate, moduleId)) {
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
						if (
							runtime.__rspack_require__.o(
								result.outdatedDependencies,
								moduleId
							)
						) {
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
			var module = runtime.moduleCache[outdatedModuleId];
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
					var module = runtime.moduleCache[moduleId];
					if (!module) continue;

					var data = {};

					// Call dispose handlers
					var disposeHandlers = module.hot._disposeHandlers;
					for (j = 0; j < disposeHandlers.length; j++) {
						disposeHandlers[j].call(null, data);
					}
					runtime.__rspack_require__.hmrD[moduleId] = data;

					module.hot.active = false;

					delete runtime.moduleCache[moduleId];

					delete outdatedDependencies[moduleId];

					for (j = 0; j < module.children.length; j++) {
						var child = runtime.moduleCache[module.children[j]];
						if (!child) continue;
						idx = child.parents.indexOf(moduleId);
						if (idx >= 0) {
							child.parents.splice(idx, 1);
						}
					}
				}

				var dependency;
				for (var outdatedModuleId in outdatedDependencies) {
					if (
						runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)
					) {
						module = runtime.moduleCache[outdatedModuleId];
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
					if (runtime.__rspack_require__.o(appliedUpdate, updateModuleId)) {
						runtime.installedModules[updateModuleId] =
							appliedUpdate[updateModuleId];
					}
				}

				// run new runtime modules
				for (var i = 0; i < currentUpdateRuntime.length; i++) {
					currentUpdateRuntime[i](runtime.__rspack_require__);
				}

				// call accept handlers
				for (var outdatedModuleId in outdatedDependencies) {
					if (
						runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)
					) {
						var module = runtime.moduleCache[outdatedModuleId];
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
									module: runtime.moduleCache[moduleId]
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

	runtime.__rspack_require__.hmrI.jsonp = function (moduleId, applyHandlers) {
		if (!currentUpdate) {
			currentUpdate = {};
			currentUpdateRuntime = [];
			currentUpdateRemovedChunks = [];
			applyHandlers.push(applyHandler);
		}
		if (!runtime.__rspack_require__.o(currentUpdate, moduleId)) {
			currentUpdate[moduleId] = runtime.installedModules[moduleId];
		}
	};

	// TODO: fetch is not needed
	runtime.__rspack_require__.hmrC.jsonp = function (
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
				runtime.__rspack_require__.o(installedChunks, chunkId) &&
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
