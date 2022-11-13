(function () { // runtime instance
var runtime = new Object();
self["__rspack_runtime__"] = runtime;
// mount Modules
(function () {
	runtime.installedModules = {/* __INSTALLED_MODULES__*/};
})();

// mount Chunks
(function () {
	runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
	runtime.moduleCache = {};
})();
(function () {
	runtime.checkById = function (obj, prop) {
		return Object.prototype.hasOwnProperty.call(obj, prop);
	};
})();
// mount PublicPath
(function () {
	runtime.publicPath = "/";
})();
// The require function
function __rspack_require__(moduleId) {
	var cachedModule = runtime.moduleCache[moduleId];
	if (cachedModule !== undefined) {
		return cachedModule.exports;
	}

	// Create a new module (and put it into the cache)
	var module = (runtime.moduleCache[moduleId] = {
		// no module.id needed
		// no module.loaded needed
		exports: {}
	});

	// TODO: should use runtime generator
	//---- hot require
	try {
		var execOptions = {
			id: moduleId,
			module: module,
			factory: runtime.installedModules[moduleId],
			require: __rspack_require__
		};
		module = execOptions.module;
		__rspack_require__.i.forEach(function (handler) {
			handler(execOptions);
		});
		execOptions.factory.call(
			module.exports,
			module,
			module.exports,
			execOptions.require.bind(runtime),
			runtime.__rspack_dynamic_require__ &&
				runtime.__rspack_dynamic_require__.bind(runtime),
			runtime
		);
	} catch (error) {
		module.error = error;
		throw error;
	}

	//------ other
	// this.installedModules[moduleId](
	// 	module,
	// 	module.exports,
	// 	this.__rspack_require__.bind(this),
	// 	this.__rspack_dynamic_require__ &&
	// 		this.__rspack_dynamic_require__.bind(this),
	//  runtime,
	// );

	return module.exports;
}

// mount require function
(function () {
	runtime.__rspack_require__ = __rspack_require__;
	// module execution interceptor
	runtime.__rspack_require__.i = [];
	// hasOwnProperty shorthand
	runtime.__rspack_require__.o = (obj, prop) =>
		Object.prototype.hasOwnProperty.call(obj, prop);
})();
// The register function
function __rspack_register__(chunkIds, modules, callback) {
	if (
		chunkIds.some(
			function (id) {
				return this.installedChunks[id] !== 0;
			}.bind(this)
		)
	) {
		for (moduleId in modules) {
			if (this.checkById(modules, moduleId)) {
				this.installedModules[moduleId] = modules[moduleId];
			}
		}
		if (callback) callback(this.__rspack_require__);
	}
	for (var i = 0; i < chunkIds.length; i++) {
		chunkId = chunkIds[i];
		if (
			this.checkById(this.installedChunks, chunkId) &&
			this.installedChunks[chunkId]
		) {
			this.installedChunks[chunkId][0]();
		}
		this.installedChunks[chunkId] = 0;
	}
}

// mount register function
(function () {
	runtime.__rspack_register__ = __rspack_register__;
})();
(function(){
runtime.__rspack_require__.chunkId = 'main'})();(function(){
runtime.__rspack_require__.p = '/'})();// hot runtime
(function () {
	var currentModuleData = {};
	var installedModules = runtime.moduleCache;

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

	runtime.__rspack_require__.hmrD = currentModuleData;
	runtime.__rspack_require__.i.push(function (options) {
		var module = options.module;
		var require = createRequire(options.require, options.id);
		module.hot = createModuleHotObject(options.id, module);
		module.parents = currentParents;
		module.children = [];
		currentParents = [];
		options.require = require;
	});

	runtime.__rspack_require__.hmrC = {};
	// TODO: useless
	runtime.__rspack_require__.hmrI = {};

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
				runtime.__rspack_require__(moduleId);
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
						// TODO: useless
						currentUpdateApplyHandlers = [];
						Object.keys(runtime.__rspack_require__.hmrI).forEach(function (
							key
						) {
							runtime.__rspack_require__.hmrI[key](
								moduleId,
								currentUpdateApplyHandlers
							);
						});
						setStatus("ready");
						break;
					case "ready":
						Object.keys(runtime.__rspack_require__.hmrI).forEach(function (
							key
						) {
							runtime.__rspack_require__.hmrI[key](
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
			.then(runtime.__rspack_require__.hmrM)
			.then(function (update) {
				if (!update) {
					return setStatus(applyInvalidatedModules() ? "ready" : "idle").then(
						function () {
							return null;
						}
					);
				}

				return setStatus("prepare").then(function () {
					// var updatedModules = [];
					// TODO: updatedModule should removed after hash
					var updatedModules = update.updatedModule;
					currentUpdateApplyHandlers = [];
					return Promise.all(
						// TODO: update.c, .r, .m is useless now.
						Object.keys(runtime.__rspack_require__.hmrC).reduce(function (
							promises,
							key
						) {
							runtime.__rspack_require__.hmrC[key](
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
				return r.errors;
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
			Object.keys(runtime.__rspack_require__.hmrI).forEach(function (key) {
				queuedInvalidatedModules.forEach(function (moduleId) {
					runtime.__rspack_require__.hmrI[key](
						moduleId,
						currentUpdateApplyHandlers
					);
				});
			});
			queuedInvalidatedModules = undefined;
			return true;
		}
	}
})();
(() => {
	var inProgress = {};
	// data-webpack is not used as build has no uniqueName
	// loadScript function to load a script via script tag
	runtime.__rspack_require__.l = (content, done, key, chunkId) => {
		// if (inProgress[url]) {
		// 	inProgress[url].push(done);
		// 	return;
		// }
		var script, needAttach;
		if (key !== undefined) {
			var scripts = document.getElementsByTagName("script");
			for (var i = 0; i < scripts.length; i++) {
				var s = scripts[i];
				// if (s.getAttribute("src") == url) {
				// 	script = s;
				// 	break;
				// }
				if (s.text == content) {
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
			script.id = "hot-script";
			// if (__webpack_require__.nc) {
			// 	script.setAttribute("nonce", __webpack_require__.nc);
			// }

			// script.src = url;
			script.text = content;
		}
		// inProgress[url] = [done];
		inProgress[content] = [done];
		var onScriptComplete = (prev, event) => {
			// avoid mem leaks in IE.
			script.onerror = script.onload = null;
			clearTimeout(timeout);
			// var doneFns = inProgress[url];
			// delete inProgress[url];
			var doneFns = inProgress[content];
			delete inProgress[content];
			script.parentNode && script.parentNode.removeChild(script);
			doneFns && doneFns.forEach(fn => fn(event));
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
	var installedChunks = (runtime.__rspack_require__.hmrS_jsonp = runtime
		.__rspack_require__.hmrS_jsonp || {
		main: 0
	});

	var currentUpdatedModulesList;
	var waitingUpdateResolves = {};
	function loadUpdateChunk(chunkId, updatedModulesList, content) {
		currentUpdatedModulesList = updatedModulesList;
		return new Promise((resolve, reject) => {
			waitingUpdateResolves[chunkId] = resolve;
			// start update chunk loading
			// var url = __webpack_require__.p + __webpack_require__.hu(chunkId);
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
			runtime.__rspack_require__.l(content, loadingEnded);
		});
	}

	self["hotUpdate"] = (chunkId, moreModules, runtime) => {
		for (var moduleId in moreModules) {
			if (__rspack_runtime__.__rspack_require__.o(moreModules, moduleId)) {
				currentUpdate[moduleId] = moreModules[moduleId];
				if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId);
			}
		}
		if (runtime) currentUpdateRuntime.push(runtime);
		if (waitingUpdateResolves[chunkId]) {
			waitingUpdateResolves[chunkId]();
			waitingUpdateResolves[chunkId] = undefined;
			var tag = document.getElementById("hot-script");
			tag && tag.parentNode && tag.parentNode.removeChild(tag);
		}
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
		// updatedModulesList,
		updatedModules
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
				// TODO: use load script after hash.
				// promises.push(loadUpdateChunk(chunkId, updatedModulesList));
				var updatedModulesList = [updatedModules.uri];
				promises.push(
					loadUpdateChunk(chunkId, updatedModulesList, updatedModules.content)
				);

				currentUpdateChunks[chunkId] = true;
			} else {
				currentUpdateChunks[chunkId] = false;
			}
		});
	};
})();
(function () {
	runtime.installedCssChunks = {};
})();

(function () {
	runtime.chunkHashData = {
		js: {"main": "",},
		css: {}
	};
})();

(function () {
	runtime.setChunkHashData = function (chunkId, hash, type) {
		return (this.chunkHashData[type][chunkId] = hash);
	};
})();

(function () {
	runtime.__rspack_has_dynamic_chunk__ = function (chunkId, type) {
		return true;
		return Boolean(
			this.chunkHashData &&
				this.chunkHashData[type] &&
				typeof this.chunkHashData[type][chunkId] !== "undefined"
		);
	};
})();
(function () {
	runtime.__rspack_get_dynamic_chunk_url__ = function (chunkId, type) {
		return chunkId + "" + "." + type;
	};
})();
function __rspack_dynamic_require__(chunkIds) {
	return Promise.all(
		chunkIds
			.map(
				function (chunkId) {
					return Object.keys(this)
						.filter(function (key) {
							return key.indexOf("rspack_load_dynamic") > 0;
						})
						.reduce(
							function (promises, key) {
								this[key](chunkId, promises);
								return promises;
							}.bind(this),
							[]
						);
				}.bind(this)
			)
			.reduce(function (prev, cur) {
				return prev.concat(cur);
			}, [])
	);
}

// mount register dynamic require
(function () {
	runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
})();
var inProgress = {};
function load_script(url, done, key) {
	var dataWebpackPrefix = "rspack-test:";
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
				s.getAttribute("data-rspack") == dataWebpackPrefix + key
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
		script.setAttribute("data-rspack", dataWebpackPrefix + key);
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
		onScriptComplete.bind(null, undefined, { type: "timeout", target: script }),
		120000
	);
	script.onerror = onScriptComplete.bind(null, script.onerror);
	script.onload = onScriptComplete.bind(null, script.onload);
	needAttach && document.head.appendChild(script);
}

function __rspack_load_dynamic_js__(chunkId, promises) {
	var runtime = this;
	var installedChunkData = this.checkById(this.installedChunks, chunkId)
		? this.installedChunks[chunkId]
		: undefined;
	if (installedChunkData !== 0) {
		if (installedChunkData) {
			promises.push(installedChunkData[2]);
		} else {
			var promise = new Promise(
				function (resolve, reject) {
					installedChunkData = this.installedChunks[chunkId] = [
						resolve,
						reject
					];
				}.bind(this)
			);
			promises.push((installedChunkData[2] = promise));
			var url =
				this.publicPath + this.__rspack_get_dynamic_chunk_url__(chunkId, "js");
			var error = new Error();
			var loadingEnded = function (event) {
				if (runtime.checkById(runtime.installedChunks, chunkId)) {
					installedChunkData = runtime.installedChunks[chunkId];
					if (installedChunkData !== 0)
						runtime.installedChunks[chunkId] = undefined;
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
			load_script(url, loadingEnded, "chunk-" + chunkId);
		}
	}
}

// mount load dynamic js
(function () {
	runtime.__rspack_load_dynamic_js__ = __rspack_load_dynamic_js__;
})();
function load_style(chunkId, href, fullhref, resolve, reject) {
	var existingLinkTags = document.getElementsByTagName("link");
	for (var i = 0; i < existingLinkTags.length; i++) {
		var tag = existingLinkTags[i];
		var dataHref = tag.getAttribute("data-href") || tag.getAttribute("href");
		if (
			tag.rel === "stylesheet" &&
			(dataHref === href || dataHref === fullhref)
		)
			return resolve();
	}
	var existingStyleTags = document.getElementsByTagName("style");
	for (var i = 0; i < existingStyleTags.length; i++) {
		var tag = existingStyleTags[i];
		var dataHref = tag.getAttribute("data-href");
		if (dataHref === href || dataHref === fullhref) return resolve();
	}
	var linkTag = document.createElement("link");
	linkTag.rel = "stylesheet";
	linkTag.type = "text/css";
	var onLinkComplete = function (event) {
		linkTag.onerror = linkTag.onload = null;
		if (event.type === "load") {
			resolve();
		} else {
			var errorType = event && (event.type === "load" ? "missing" : event.type);
			var realHref = (event && event.target && event.target.href) || fullhref;
			var err = new Error(
				"Loading CSS chunk " + chunkId + " failed.\n(" + realHref + ")"
			);
			err.code = "CSS_CHUNK_LOAD_FAILED";
			err.type = errorType;
			err.request = realHref;
			linkTag.parentNode.removeChild(linkTag);
			reject(err);
		}
	};
	linkTag.onerror = linkTag.onload = onLinkComplete;
	linkTag.href = fullhref;
	document.head.appendChild(linkTag);
	return linkTag;
}

function __rspack_load_dynamic_css__(chunkId, promises) {
	var installedChunkData = this.installedCssChunks[chunkId];
	if (installedChunkData) {
		promises.push(installedChunkData);
	} else if (
		installedChunkData !== 0 &&
		this.__rspack_has_dynamic_chunk__(chunkId, "css")
	) {
		var href = this.__rspack_get_dynamic_chunk_url__(chunkId, "css");
		var fullhref = this.publicPath + href;
		promises.push(
			(installedChunkData = new Promise(function (resolve, reject) {
				load_style(chunkId, href, fullhref, resolve, reject);
			}).then(
				function () {
					installedChunkData = 0;
				},
				function (e) {
					installedChunkData = undefined;
					throw e;
				}
			))
		);
	}
}

// mount load dynamic css
(function () {
	runtime.__rspack_load_dynamic_css__ = __rspack_load_dynamic_css__;
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

	runtime.interopRequire = function (obj, nodeInterop) {
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
	runtime.exportStar = function (from, to) {
		Object.keys(from).forEach(function (k) {
			if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k))
				Object.defineProperty(to, k, {
					enumerable: true,
					get: function () {
						return from[k];
					}
				});
		});
		return from;
	};
})();
self["__rspack_runtime__"].__rspack_register__(["main"], {
"./images/file.jpg": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
module.exports = "data:image/jpeg;base64,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";},
"./images/file.png": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
module.exports = "/assets/file.png";},
"./images/file.svg": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
module.exports = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA2MDAgNjAwIj48dGl0bGU+aWNvbi1zcXVhcmUtc21hbGw8L3RpdGxlPjxwYXRoIGZpbGw9IiNGRkYiIGQ9Ik0zMDAgLjFMNTY1IDE1MHYyOTkuOUwzMDAgNTk5LjggMzUgNDQ5LjlWMTUweiIvPjxwYXRoIGZpbGw9IiM4RUQ2RkIiIGQ9Ik01MTcuNyA0MzkuNUwzMDguOCA1NTcuOHYtOTJMNDM5IDM5NC4xbDc4LjcgNDUuNHptMTQuMy0xMi45VjE3OS40bC03Ni40IDQ0LjF2MTU5bDc2LjQgNDQuMXpNODEuNSA0MzkuNWwyMDguOSAxMTguMnYtOTJsLTEzMC4yLTcxLjYtNzguNyA0NS40em0tMTQuMy0xMi45VjE3OS40bDc2LjQgNDQuMXYxNTlsLTc2LjQgNDQuMXptOC45LTI2My4yTDI5MC40IDQyLjJ2ODlsLTEzNy4zIDc1LjUtMS4xLjYtNzUuOS00My45em00NDYuOSAwTDMwOC44IDQyLjJ2ODlMNDQ2IDIwNi44bDEuMS42IDc1LjktNDR6Ii8+PHBhdGggZmlsbD0iIzFDNzhDMCIgZD0iTTI5MC40IDQ0NC44TDE2MiAzNzQuMVYyMzQuMmwxMjguNCA3NC4xdjEzNi41em0xOC40IDBsMTI4LjQtNzAuNnYtMTQwbC0xMjguNCA3NC4xdjEzNi41ek0yOTkuNiAzMDN6bS0xMjktODVsMTI5LTcwLjlMNDI4LjUgMjE4bC0xMjguOSA3NC40LTEyOS03NC40eiIvPjwvc3ZnPgo=";},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _filePng = __rspack_runtime__.interopRequire(__rspack_require__("./images/file.png"));
const _fileJpg = __rspack_runtime__.interopRequire(__rspack_require__("./images/file.jpg"));
const _fileSvg = __rspack_runtime__.interopRequire(__rspack_require__("./images/file.svg"));
const container = document.createElement("div");
Object.assign(container.style, {
    display: "flex",
    justifyContent: "center"
});
document.body.appendChild(container);
function createImageElement(title, src) {
    const div = document.createElement("div");
    div.style.textAlign = "center";
    const h2 = document.createElement("h2");
    h2.textContent = title;
    div.appendChild(h2);
    const img = document.createElement("img");
    img.setAttribute("src", src);
    img.setAttribute("width", "150");
    div.appendChild(img);
    container.appendChild(div);
}
[
    _filePng.default,
    _fileJpg.default,
    _fileSvg.default
].forEach((src)=>{
    createImageElement(src.split(".").pop(), src);
});
},
});self["__rspack_runtime__"].__rspack_require__("./index.js"); })();