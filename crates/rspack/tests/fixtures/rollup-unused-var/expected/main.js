(function() {
    var runtime = new Object();
    self["__rspack_runtime__"] = runtime;
    (function() {
        runtime.installedModules = {};
    })();
    (function() {
        runtime.installedChunks = {};
    })();
    (function() {
        runtime.moduleCache = {};
    })();
    (function() {
        runtime.checkById = function(obj, prop) {
            return Object.prototype.hasOwnProperty.call(obj, prop);
        };
    })();
    (function() {
        runtime.publicPath = "/";
    })();
    function __rspack_require__(moduleId1) {
        var cachedModule = runtime.moduleCache[moduleId1];
        if (void 0 !== cachedModule) return cachedModule.exports;
        var module = runtime.moduleCache[moduleId1] = {
            exports: {}
        };
        try {
            var execOptions = {
                id: moduleId1,
                module: module,
                factory: runtime.installedModules[moduleId1],
                require: __rspack_require__
            };
            module = execOptions.module;
            __rspack_require__.i.forEach(function(handler) {
                handler(execOptions);
            });
            execOptions.factory.call(module.exports, module, module.exports, execOptions.require.bind(runtime), runtime.__rspack_dynamic_require__ && runtime.__rspack_dynamic_require__.bind(runtime), runtime);
        } catch (error) {
            module.error = error;
            throw error;
        }
        return module.exports;
    }
    (function() {
        runtime.__rspack_require__ = __rspack_require__;
        runtime.__rspack_require__.i = [];
        runtime.__rspack_require__.o = (obj, prop)=>Object.prototype.hasOwnProperty.call(obj, prop);
    })();
    function __rspack_register__(chunkIds, modules, callback) {
        if (chunkIds.some((function(id) {
            return 0 !== this.installedChunks[id];
        }).bind(this))) {
            for(moduleId in modules)if (this.checkById(modules, moduleId)) this.installedModules[moduleId] = modules[moduleId];
            if (callback) callback(this.__rspack_require__);
        }
        for(var i = 0; i < chunkIds.length; i++){
            chunkId = chunkIds[i];
            if (this.checkById(this.installedChunks, chunkId) && this.installedChunks[chunkId]) this.installedChunks[chunkId][0]();
            this.installedChunks[chunkId] = 0;
        }
    }
    (function() {
        runtime.__rspack_register__ = __rspack_register__;
    })();
    (function() {
        runtime.__rspack_require__.chunkId = "main";
    })();
    (function() {
        var currentModuleData = {};
        var installedModules = runtime.moduleCache;
        var currentChildModule;
        var currentParents = [];
        var registeredStatusHandlers = [];
        var currentStatus = "idle";
        var blockingPromises = 0;
        var blockingPromisesWaiting = [];
        var currentUpdateApplyHandlers;
        var queuedInvalidatedModules;
        runtime.__rspack_require__.hmrD = currentModuleData;
        runtime.__rspack_require__.i.push(function(options) {
            var module = options.module;
            var require = createRequire(options.require, options.id);
            module.hot = createModuleHotObject(options.id, module);
            module.parents = currentParents;
            module.children = [];
            currentParents = [];
            options.require = require;
        });
        runtime.__rspack_require__.hmrC = {};
        runtime.__rspack_require__.hmrI = {};
        function createRequire(require, moduleId1) {
            var me = installedModules[moduleId1];
            if (!me) return require;
            var fn = function(request) {
                if (me.hot.active) {
                    if (installedModules[request]) {
                        var parents = installedModules[request].parents;
                        if (-1 === parents.indexOf(moduleId1)) parents.push(moduleId1);
                    } else {
                        currentParents = [
                            moduleId1
                        ];
                        currentChildModule = request;
                    }
                    if (-1 === me.children.indexOf(request)) me.children.push(request);
                } else {
                    console.log("[HMR] unexpected require(" + request + ") from disposed module " + moduleId1);
                    currentParents = [];
                }
                return require(request);
            };
            var createPropertyDescriptor = function(name) {
                return {
                    configurable: true,
                    enumerable: true,
                    get: function() {
                        return require[name];
                    },
                    set: function(value) {
                        require[name] = value;
                    }
                };
            };
            for(var name in require)if (Object.prototype.hasOwnProperty.call(require, name) && "e" !== name) Object.defineProperty(fn, name, createPropertyDescriptor(name));
            fn.e = function(chunkId1) {
                return trackBlockingPromise(require.e(chunkId1));
            };
            return fn;
        }
        function createModuleHotObject(moduleId1, me) {
            var _main = currentChildModule !== moduleId1;
            var hot = {
                _acceptedDependencies: {},
                _acceptedErrorHandlers: {},
                _declinedDependencies: {},
                _selfAccepted: false,
                _selfDeclined: false,
                _selfInvalidated: false,
                _disposeHandlers: [],
                _main: _main,
                _requireSelf: function() {
                    currentParents = me.parents.slice();
                    currentChildModule = _main ? void 0 : moduleId1;
                    runtime.__rspack_require__(moduleId1);
                },
                active: true,
                accept: function(dep, callback, errorHandler) {
                    if (void 0 === dep) hot._selfAccepted = true;
                    else if ("function" == typeof dep) hot._selfAccepted = dep;
                    else if ("object" == typeof dep && null !== dep) for(var i = 0; i < dep.length; i++){
                        hot._acceptedDependencies[dep[i]] = callback || function() {};
                        hot._acceptedErrorHandlers[dep[i]] = errorHandler;
                    }
                    else {
                        hot._acceptedDependencies[dep] = callback || function() {};
                        hot._acceptedErrorHandlers[dep] = errorHandler;
                    }
                },
                decline: function(dep) {
                    if (void 0 === dep) hot._selfDeclined = true;
                    else if ("object" == typeof dep && null !== dep) for(var i = 0; i < dep.length; i++)hot._declinedDependencies[dep[i]] = true;
                    else hot._declinedDependencies[dep] = true;
                },
                dispose: function(callback) {
                    hot._disposeHandlers.push(callback);
                },
                addDisposeHandler: function(callback) {
                    hot._disposeHandlers.push(callback);
                },
                removeDisposeHandler: function(callback) {
                    var idx = hot._disposeHandlers.indexOf(callback);
                    if (idx > 0) hot._disposeHandlers.splice(idx, 1);
                },
                invalidate: function() {
                    this._selfInvalidated = true;
                    switch(currentStatus){
                        case "idle":
                            currentUpdateApplyHandlers = [];
                            Object.keys(runtime.__rspack_require__.hmrI).forEach(function(key) {
                                runtime.__rspack_require__.hmrI[key](moduleId1, currentUpdateApplyHandlers);
                            });
                            setStatus("ready");
                            break;
                        case "ready":
                            Object.keys(runtime.__rspack_require__.hmrI).forEach(function(key) {
                                runtime.__rspack_require__.hmrI[key](moduleId1, currentUpdateApplyHandlers);
                            });
                            break;
                        case "prepare":
                        case "check":
                        case "dispose":
                        case "apply":
                            (queuedInvalidatedModules = queuedInvalidatedModules || []).push(moduleId1);
                            break;
                        default:
                            break;
                    }
                },
                check: hotCheck,
                apply: hotApply,
                status: function(l) {
                    if (!l) return currentStatus;
                    registeredStatusHandlers.push(l);
                },
                addStatusHandler: function(l) {
                    registeredStatusHandlers.push(l);
                },
                removeStatusHandler: function(l) {
                    var idx = registeredStatusHandlers.indexOf(l);
                    if (idx >= 0) registeredStatusHandlers.splice(idx, 1);
                },
                data: currentModuleData[moduleId1]
            };
            currentChildModule = void 0;
            return hot;
        }
        function setStatus(newStats) {
            currentStatus = newStats;
            var results = [];
            for(var i = 0; i < registeredStatusHandlers.length; i++)results[i] = registeredStatusHandlers[i].call(null, newStats);
            return Promise.all(results);
        }
        function unblock() {
            if (0 === --blockingPromises) setStatus("ready").then(function() {
                if (0 === blockingPromises) {
                    var list = blockingPromisesWaiting;
                    blockingPromisesWaiting = [];
                    for(var i = 0; i < list.length; i++)list[i]();
                }
            });
        }
        function trackBlockingPromise(promise) {
            switch(currentStatus){
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
            if (0 === blockingPromises) return fn();
            return new Promise(function(resolve) {
                blockingPromisesWaiting.push(function() {
                    resolve(fn());
                });
            });
        }
        function hotCheck(applyOnUpdate) {
            if ("idle" !== currentStatus) throw new Error("check() is only allowed in idle status");
            return setStatus("check").then(runtime.__rspack_require__.hmrM).then(function(update) {
                if (!update) return setStatus(applyInvalidatedModules() ? "ready" : "idle").then(function() {
                    return null;
                });
                return setStatus("prepare").then(function() {
                    var updatedModules = update.updatedModule;
                    currentUpdateApplyHandlers = [];
                    return Promise.all(Object.keys(runtime.__rspack_require__.hmrC).reduce(function(promises, key) {
                        runtime.__rspack_require__.hmrC[key](update.c, update.r, update.m, promises, currentUpdateApplyHandlers, updatedModules);
                        return promises;
                    }, [])).then(function() {
                        return waitForBlockingPromises(function() {
                            if (applyOnUpdate) return internalApply(applyOnUpdate);
                            return setStatus("ready").then(function() {
                                return updatedModules;
                            });
                        });
                    });
                });
            });
        }
        function hotApply(options) {
            if ("ready" !== currentStatus) return Promise.resolve().then(function() {
                throw Error("apply() is only allowed in ready status (state: " + currentStatus + ")");
            });
            return internalApply(options);
        }
        function internalApply(options) {
            options = options || {};
            applyInvalidatedModules();
            var results = currentUpdateApplyHandlers.map(function(handler) {
                return handler(options);
            });
            currentUpdateApplyHandlers = void 0;
            var errors = results.map(function(r) {
                return r.errors;
            }).filter(Boolean);
            if (errors.length > 0) return setStatus("abort").then(function() {
                throw errors[0];
            });
            var disposePromise = setStatus("dispose");
            results.forEach(function(result) {
                if (result.dispose) result.dispose();
            });
            var applyPromise = setStatus("apply");
            var error;
            var reportError = function(err) {
                if (!error) error = err;
            };
            var outdatedModules = [];
            results.forEach(function(result) {
                if (result.apply) {
                    var modules = result.apply(reportError);
                    if (modules) for(var i = 0; i < modules.length; i++)outdatedModules.push(modules[i]);
                }
            });
            return Promise.all([
                disposePromise,
                applyPromise
            ]).then(function() {
                if (error) return setStatus("fail").then(function() {
                    throw error;
                });
                if (queuedInvalidatedModules) return internalApply(options).then(function(list) {
                    outdatedModules.forEach(function(moduleId1) {
                        if (list.indexOf(moduleId1) < 0) list.push(moduleId1);
                    });
                    return list;
                });
                return setStatus("idle").then(function() {
                    return outdatedModules;
                });
            });
        }
        function applyInvalidatedModules() {
            if (queuedInvalidatedModules) {
                if (!currentUpdateApplyHandlers) currentUpdateApplyHandlers = [];
                Object.keys(runtime.__rspack_require__.hmrI).forEach(function(key) {
                    queuedInvalidatedModules.forEach(function(moduleId1) {
                        runtime.__rspack_require__.hmrI[key](moduleId1, currentUpdateApplyHandlers);
                    });
                });
                queuedInvalidatedModules = void 0;
                return true;
            }
        }
    })();
    (()=>{
        var inProgress = {};
        runtime.__rspack_require__.l = (content, done, key, chunkId1)=>{
            var script, needAttach;
            if (void 0 !== key) {
                var scripts = document.getElementsByTagName("script");
                for(var i = 0; i < scripts.length; i++){
                    var s = scripts[i];
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
                script.text = content;
            }
            inProgress[content] = [
                done
            ];
            var onScriptComplete = (prev, event)=>{
                script.onerror = script.onload = null;
                clearTimeout(timeout);
                var doneFns = inProgress[content];
                delete inProgress[content];
                script.parentNode && script.parentNode.removeChild(script);
                doneFns && doneFns.forEach((fn)=>fn(event));
                if (prev) return prev(event);
            };
            var timeout = setTimeout(onScriptComplete.bind(null, void 0, {
                type: "timeout",
                target: script
            }), 120000);
            script.onerror = onScriptComplete.bind(null, script.onerror);
            script.onload = onScriptComplete.bind(null, script.onload);
            needAttach && document.head.appendChild(script);
        };
    })();
    (function() {
        var installedChunks = runtime.__rspack_require__.hmrS_jsonp = runtime.__rspack_require__.hmrS_jsonp || {
            main: 0
        };
        var currentUpdatedModulesList;
        var waitingUpdateResolves = {};
        function loadUpdateChunk(chunkId1, updatedModulesList, content) {
            currentUpdatedModulesList = updatedModulesList;
            return new Promise((resolve, reject)=>{
                waitingUpdateResolves[chunkId1] = resolve;
                var error = new Error();
                var loadingEnded = (event)=>{
                    if (waitingUpdateResolves[chunkId1]) {
                        waitingUpdateResolves[chunkId1] = void 0;
                        var errorType = event && ("load" === event.type ? "missing" : event.type);
                        var realSrc = event && event.target && event.target.src;
                        error.message = "Loading hot update chunk " + chunkId1 + " failed.\n(" + errorType + ": " + realSrc + ")";
                        error.name = "ChunkLoadError";
                        error.type = errorType;
                        error.request = realSrc;
                        reject(error);
                    }
                };
                runtime.__rspack_require__.l(content, loadingEnded);
            });
        }
        self["hotUpdate"] = (chunkId1, moreModules, runtime)=>{
            for(var moduleId1 in moreModules)if (__rspack_runtime__.__rspack_require__.o(moreModules, moduleId1)) {
                currentUpdate[moduleId1] = moreModules[moduleId1];
                if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId1);
            }
            if (runtime) currentUpdateRuntime.push(runtime);
            if (waitingUpdateResolves[chunkId1]) {
                waitingUpdateResolves[chunkId1]();
                waitingUpdateResolves[chunkId1] = void 0;
                var tag = document.getElementById("hot-script");
                tag && tag.parentNode && tag.parentNode.removeChild(tag);
            }
        };
        var currentUpdateChunks;
        var currentUpdate;
        var currentUpdateRemovedChunks;
        var currentUpdateRuntime;
        function applyHandler(options) {
            currentUpdateChunks = void 0;
            function getAffectedModuleEffects(updateModuleId) {
                var outdatedModules = [
                    updateModuleId
                ];
                var outdatedDependencies = {};
                var queue = outdatedModules.map(function(id) {
                    return {
                        chain: [
                            id
                        ],
                        id: id
                    };
                });
                while(queue.length > 0){
                    var queueItem = queue.pop();
                    var moduleId1 = queueItem.id;
                    var chain = queueItem.chain;
                    var module = runtime.moduleCache[moduleId1];
                    if (!module || module.hot._selfAccepted && !module.hot._selfInvalidated) continue;
                    if (module.hot._selfDeclined) return {
                        type: "self-declined",
                        chain: chain,
                        moduleId: moduleId1
                    };
                    if (module.hot._main) return {
                        type: "unaccepted",
                        chain: chain,
                        moduleId: moduleId1
                    };
                    for(var i = 0; i < module.parents.length; i++){
                        var parentId = module.parents[i];
                        var parent = runtime.moduleCache[parentId];
                        if (!!parent) {
                            if (parent.hot._declinedDependencies[moduleId1]) return {
                                type: "declined",
                                chain: chain.concat([
                                    parentId
                                ]),
                                moduleId: moduleId1,
                                parentId: parentId
                            };
                            if (-1 === outdatedModules.indexOf(parentId)) {
                                if (parent.hot._acceptedDependencies[moduleId1]) {
                                    if (!outdatedDependencies[parentId]) outdatedDependencies[parentId] = [];
                                    addAllToSet(outdatedDependencies[parentId], [
                                        moduleId1
                                    ]);
                                    continue;
                                }
                                delete outdatedDependencies[parentId];
                                outdatedModules.push(parentId);
                                queue.push({
                                    chain: chain.concat([
                                        parentId
                                    ]),
                                    id: parentId
                                });
                            }
                        }
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
                for(var i = 0; i < b.length; i++){
                    var item = b[i];
                    if (-1 === a.indexOf(item)) a.push(item);
                }
            }
            var outdatedDependencies = {};
            var outdatedModules = [];
            var appliedUpdate = {};
            var warnUnexpectedRequire = function(module) {
                console.warn("[HMR] unexpected require(" + module.id + ") to disposed module");
            };
            for(var moduleId1 in currentUpdate)if (runtime.__rspack_require__.o(currentUpdate, moduleId1)) {
                var newModuleFactory = currentUpdate[moduleId1];
                var result;
                result = newModuleFactory ? getAffectedModuleEffects(moduleId1) : {
                    type: "disposed",
                    moduleId: moduleId1
                };
                var abortError = false;
                var doApply = false;
                var doDispose = false;
                var chainInfo = "";
                if (result.chain) chainInfo = "\nUpdate propagation: " + result.chain.join(" -> ");
                switch(result.type){
                    case "self-declined":
                        if (options.onDeclined) options.onDeclined(result);
                        if (!options.ignoreDeclined) abortError = new Error("Aborted because of self decline: " + result.moduleId + chainInfo);
                        break;
                    case "declined":
                        if (options.onDeclined) options.onDeclined(result);
                        if (!options.ignoreDeclined) abortError = new Error("Aborted because of declined dependency: " + result.moduleId + " in " + result.parentId + chainInfo);
                        break;
                    case "unaccepted":
                        if (options.onUnaccepted) options.onUnaccepted(result);
                        if (!options.ignoreUnaccepted) abortError = new Error("Aborted because " + moduleId1 + " is not accepted" + chainInfo);
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
                if (abortError) return {
                    error: abortError
                };
                if (doApply) {
                    appliedUpdate[moduleId1] = newModuleFactory;
                    addAllToSet(outdatedModules, result.outdatedModules);
                    for(moduleId1 in result.outdatedDependencies)if (runtime.__rspack_require__.o(result.outdatedDependencies, moduleId1)) {
                        if (!outdatedDependencies[moduleId1]) outdatedDependencies[moduleId1] = [];
                        addAllToSet(outdatedDependencies[moduleId1], result.outdatedDependencies[moduleId1]);
                    }
                }
                if (doDispose) {
                    addAllToSet(outdatedModules, [
                        result.moduleId
                    ]);
                    appliedUpdate[moduleId1] = warnUnexpectedRequire;
                }
            }
            currentUpdate = void 0;
            var outdatedSelfAcceptedModules = [];
            for(var j = 0; j < outdatedModules.length; j++){
                var outdatedModuleId = outdatedModules[j];
                var module = runtime.moduleCache[outdatedModuleId];
                if (module && (module.hot._selfAccepted || module.hot._main) && appliedUpdate[outdatedModuleId] !== warnUnexpectedRequire && !module.hot._selfInvalidated) outdatedSelfAcceptedModules.push({
                    module: outdatedModuleId,
                    require: module.hot._requireSelf,
                    errorHandler: module.hot._selfAccepted
                });
            }
            var moduleOutdatedDependencies;
            return {
                dispose: function() {
                    currentUpdateRemovedChunks.forEach(function(chunkId1) {
                        delete installedChunks[chunkId1];
                    });
                    currentUpdateRemovedChunks = void 0;
                    var idx;
                    var queue = outdatedModules.slice();
                    while(queue.length > 0){
                        var moduleId1 = queue.pop();
                        var module = runtime.moduleCache[moduleId1];
                        if (!module) continue;
                        var data = {};
                        var disposeHandlers = module.hot._disposeHandlers;
                        for(j = 0; j < disposeHandlers.length; j++)disposeHandlers[j].call(null, data);
                        runtime.__rspack_require__.hmrD[moduleId1] = data;
                        module.hot.active = false;
                        delete runtime.moduleCache[moduleId1];
                        delete outdatedDependencies[moduleId1];
                        for(j = 0; j < module.children.length; j++){
                            var child = runtime.moduleCache[module.children[j]];
                            if (!!child) {
                                idx = child.parents.indexOf(moduleId1);
                                if (idx >= 0) child.parents.splice(idx, 1);
                            }
                        }
                    }
                    var dependency;
                    for(var outdatedModuleId in outdatedDependencies)if (runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)) {
                        module = runtime.moduleCache[outdatedModuleId];
                        if (module) {
                            moduleOutdatedDependencies = outdatedDependencies[outdatedModuleId];
                            for(j = 0; j < moduleOutdatedDependencies.length; j++){
                                dependency = moduleOutdatedDependencies[j];
                                idx = module.children.indexOf(dependency);
                                if (idx >= 0) module.children.splice(idx, 1);
                            }
                        }
                    }
                },
                apply: function(reportError) {
                    for(var updateModuleId in appliedUpdate)if (runtime.__rspack_require__.o(appliedUpdate, updateModuleId)) runtime.installedModules[updateModuleId] = appliedUpdate[updateModuleId];
                    for(var i = 0; i < currentUpdateRuntime.length; i++)currentUpdateRuntime[i](runtime.__rspack_require__);
                    for(var outdatedModuleId in outdatedDependencies)if (runtime.__rspack_require__.o(outdatedDependencies, outdatedModuleId)) {
                        var module = runtime.moduleCache[outdatedModuleId];
                        if (module) {
                            moduleOutdatedDependencies = outdatedDependencies[outdatedModuleId];
                            var callbacks = [];
                            var errorHandlers = [];
                            var dependenciesForCallbacks = [];
                            for(var j = 0; j < moduleOutdatedDependencies.length; j++){
                                var dependency = moduleOutdatedDependencies[j];
                                var acceptCallback = module.hot._acceptedDependencies[dependency];
                                var errorHandler = module.hot._acceptedErrorHandlers[dependency];
                                if (acceptCallback) {
                                    if (-1 !== callbacks.indexOf(acceptCallback)) continue;
                                    callbacks.push(acceptCallback);
                                    errorHandlers.push(errorHandler);
                                    dependenciesForCallbacks.push(dependency);
                                }
                            }
                            for(var k = 0; k < callbacks.length; k++)try {
                                callbacks[k].call(null, moduleOutdatedDependencies);
                            } catch (err) {
                                if ("function" == typeof errorHandlers[k]) try {
                                    errorHandlers[k](err, {
                                        moduleId: outdatedModuleId,
                                        dependencyId: dependenciesForCallbacks[k]
                                    });
                                } catch (err2) {
                                    if (options.onErrored) options.onErrored({
                                        type: "accept-error-handler-errored",
                                        moduleId: outdatedModuleId,
                                        dependencyId: dependenciesForCallbacks[k],
                                        error: err2,
                                        originalError: err
                                    });
                                    if (!options.ignoreErrored) {
                                        reportError(err2);
                                        reportError(err);
                                    }
                                }
                                else {
                                    if (options.onErrored) options.onErrored({
                                        type: "accept-errored",
                                        moduleId: outdatedModuleId,
                                        dependencyId: dependenciesForCallbacks[k],
                                        error: err
                                    });
                                    if (!options.ignoreErrored) reportError(err);
                                }
                            }
                        }
                    }
                    for(var o = 0; o < outdatedSelfAcceptedModules.length; o++){
                        var item = outdatedSelfAcceptedModules[o];
                        var moduleId1 = item.module;
                        try {
                            item.require(moduleId1);
                        } catch (err1) {
                            if ("function" == typeof item.errorHandler) try {
                                item.errorHandler(err1, {
                                    moduleId: moduleId1,
                                    module: runtime.moduleCache[moduleId1]
                                });
                            } catch (err21) {
                                if (options.onErrored) options.onErrored({
                                    type: "self-accept-error-handler-errored",
                                    moduleId: moduleId1,
                                    error: err21,
                                    originalError: err1
                                });
                                if (!options.ignoreErrored) {
                                    reportError(err21);
                                    reportError(err1);
                                }
                            }
                            else {
                                if (options.onErrored) options.onErrored({
                                    type: "self-accept-errored",
                                    moduleId: moduleId1,
                                    error: err1
                                });
                                if (!options.ignoreErrored) reportError(err1);
                            }
                        }
                    }
                    return outdatedModules;
                }
            };
        }
        runtime.__rspack_require__.hmrI.jsonp = function(moduleId1, applyHandlers) {
            if (!currentUpdate) {
                currentUpdate = {};
                currentUpdateRuntime = [];
                currentUpdateRemovedChunks = [];
                applyHandlers.push(applyHandler);
            }
            if (!runtime.__rspack_require__.o(currentUpdate, moduleId1)) currentUpdate[moduleId1] = runtime.installedModules[moduleId1];
        };
        runtime.__rspack_require__.hmrC.jsonp = function(chunkIds, removedChunks, removedModules, promises, applyHandlers, updatedModules) {
            applyHandlers.push(applyHandler);
            currentUpdateChunks = {};
            currentUpdateRemovedChunks = removedChunks;
            currentUpdate = removedModules.reduce(function(obj, key) {
                obj[key] = false;
                return obj;
            }, {});
            currentUpdateRuntime = [];
            chunkIds.forEach(function(chunkId1) {
                if (runtime.__rspack_require__.o(installedChunks, chunkId1) && void 0 !== installedChunks[chunkId1]) {
                    var updatedModulesList = [
                        updatedModules.uri
                    ];
                    promises.push(loadUpdateChunk(chunkId1, updatedModulesList, updatedModules.content));
                    currentUpdateChunks[chunkId1] = true;
                } else currentUpdateChunks[chunkId1] = false;
            });
        };
    })();
    (function() {
        runtime.installedCssChunks = {};
    })();
    (function() {
        runtime.chunkHashData = {
            js: {
                main: ""
            },
            css: {}
        };
    })();
    (function() {
        runtime.setChunkHashData = function(chunkId1, hash, type) {
            return this.chunkHashData[type][chunkId1] = hash;
        };
    })();
    (function() {
        runtime.__rspack_has_dynamic_chunk__ = function(chunkId1, type) {
            return true;
        };
    })();
    (function() {
        runtime.__rspack_get_dynamic_chunk_url__ = function(chunkId1, type) {
            return chunkId1 + "." + type;
        };
    })();
    function __rspack_dynamic_require__(chunkIds) {
        return Promise.all(chunkIds.map((function(chunkId1) {
            return Object.keys(this).filter(function(key) {
                return key.indexOf("rspack_load_dynamic") > 0;
            }).reduce((function(promises, key) {
                this[key](chunkId1, promises);
                return promises;
            }).bind(this), []);
        }).bind(this)).reduce(function(prev, cur) {
            return prev.concat(cur);
        }, []));
    }
    (function() {
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
        if (void 0 !== key) {
            var scripts = document.getElementsByTagName("script");
            for(var i = 0; i < scripts.length; i++){
                var s = scripts[i];
                if (s.getAttribute("src") == url || s.getAttribute("data-rspack") == dataWebpackPrefix + key) {
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
        inProgress[url] = [
            done
        ];
        var onScriptComplete = function(prev, event) {
            script.onerror = script.onload = null;
            clearTimeout(timeout);
            var doneFns = inProgress[url];
            delete inProgress[url];
            script.parentNode && script.parentNode.removeChild(script);
            doneFns && doneFns.forEach(function(fn) {
                return fn(event);
            });
            if (prev) return prev(event);
        };
        var timeout = setTimeout(onScriptComplete.bind(null, void 0, {
            type: "timeout",
            target: script
        }), 120000);
        script.onerror = onScriptComplete.bind(null, script.onerror);
        script.onload = onScriptComplete.bind(null, script.onload);
        needAttach && document.head.appendChild(script);
    }
    function __rspack_load_dynamic_js__(chunkId1, promises) {
        var runtime = this;
        var installedChunkData = this.checkById(this.installedChunks, chunkId1) ? this.installedChunks[chunkId1] : void 0;
        if (0 !== installedChunkData) {
            if (installedChunkData) promises.push(installedChunkData[2]);
            else {
                var promise = new Promise((function(resolve, reject) {
                    installedChunkData = this.installedChunks[chunkId1] = [
                        resolve,
                        reject
                    ];
                }).bind(this));
                promises.push(installedChunkData[2] = promise);
                var url = this.publicPath + this.__rspack_get_dynamic_chunk_url__(chunkId1, "js");
                var error = new Error();
                var loadingEnded = function(event) {
                    if (runtime.checkById(runtime.installedChunks, chunkId1)) {
                        installedChunkData = runtime.installedChunks[chunkId1];
                        if (0 !== installedChunkData) runtime.installedChunks[chunkId1] = void 0;
                        if (installedChunkData) {
                            var errorType = event && ("load" === event.type ? "missing" : event.type);
                            var realSrc = event && event.target && event.target.src;
                            error.message = "Loading chunk " + chunkId1 + " failed.\n(" + errorType + ": " + realSrc + ")";
                            error.name = "ChunkLoadError";
                            error.type = errorType;
                            error.request = realSrc;
                            installedChunkData[1](error);
                        }
                    }
                };
                load_script(url, loadingEnded, "chunk-" + chunkId1);
            }
        }
    }
    (function() {
        runtime.__rspack_load_dynamic_js__ = __rspack_load_dynamic_js__;
    })();
    function load_style(chunkId1, href, fullhref, resolve, reject) {
        var existingLinkTags = document.getElementsByTagName("link");
        for(var i = 0; i < existingLinkTags.length; i++){
            var tag = existingLinkTags[i];
            var dataHref = tag.getAttribute("data-href") || tag.getAttribute("href");
            if ("stylesheet" === tag.rel && (dataHref === href || dataHref === fullhref)) return resolve();
        }
        var existingStyleTags = document.getElementsByTagName("style");
        for(var i = 0; i < existingStyleTags.length; i++){
            var tag = existingStyleTags[i];
            var dataHref = tag.getAttribute("data-href");
            if (dataHref === href || dataHref === fullhref) return resolve();
        }
        var linkTag = document.createElement("link");
        linkTag.rel = "stylesheet";
        linkTag.type = "text/css";
        var onLinkComplete = function(event) {
            linkTag.onerror = linkTag.onload = null;
            if ("load" === event.type) resolve();
            else {
                var errorType = event && ("load" === event.type ? "missing" : event.type);
                var realHref = event && event.target && event.target.href || fullhref;
                var err = new Error("Loading CSS chunk " + chunkId1 + " failed.\n(" + realHref + ")");
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
    function __rspack_load_dynamic_css__(chunkId1, promises) {
        var installedChunkData = this.installedCssChunks[chunkId1];
        if (installedChunkData) promises.push(installedChunkData);
        else if (0 !== installedChunkData && this.__rspack_has_dynamic_chunk__(chunkId1, "css")) {
            var href = this.__rspack_get_dynamic_chunk_url__(chunkId1, "css");
            var fullhref = this.publicPath + href;
            promises.push(installedChunkData = new Promise(function(resolve, reject) {
                load_style(chunkId1, href, fullhref, resolve, reject);
            }).then(function() {
                installedChunkData = 0;
            }, function(e) {
                installedChunkData = void 0;
                throw e;
            }));
        }
    }
    (function() {
        runtime.__rspack_load_dynamic_css__ = __rspack_load_dynamic_css__;
    })();
    (function() {
        function _getRequireCache(nodeInterop) {
            if ("function" != typeof WeakMap) return null;
            var cacheBabelInterop = new WeakMap();
            var cacheNodeInterop = new WeakMap();
            return (_getRequireCache = function(nodeInterop) {
                return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
            })(nodeInterop);
        }
        runtime.interopRequire = function(obj, nodeInterop) {
            if (!nodeInterop && obj && obj.__esModule) return obj;
            if (null === obj || "object" != typeof obj && "function" != typeof obj) return {
                default: obj
            };
            var cache = _getRequireCache(nodeInterop);
            if (cache && cache.has(obj)) return cache.get(obj);
            var newObj = {};
            var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
            for(var key in obj)if ("default" !== key && Object.prototype.hasOwnProperty.call(obj, key)) {
                var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
                if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
                else newObj[key] = obj[key];
            }
            newObj.default = obj;
            if (cache) cache.set(obj, newObj);
            return newObj;
        };
    })();
    (function() {
        runtime.exportStar = function(from, to) {
            Object.keys(from).forEach(function(k) {
                if ("default" !== k && !Object.prototype.hasOwnProperty.call(to, k)) Object.defineProperty(to, k, {
                    enumerable: true,
                    get: function() {
                        return from[k];
                    }
                });
            });
            return from;
        };
    })();
    self["__rspack_runtime__"].__rspack_register__([
        "main"
    ], {
        "./foo.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__1) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            Object.defineProperty(exports, "foo", {
                enumerable: true,
                get: ()=>foo
            });
            var foo = "lol";
        },
        "./index.js": function(module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__1) {
            "use strict";
            Object.defineProperty(exports, "__esModule", {
                value: true
            });
            const _fooJs = __rspack_require__("./foo.js");
            console.log(_fooJs.foo);
        }
    });
    self["__rspack_runtime__"].__rspack_require__("./index.js");
})();
