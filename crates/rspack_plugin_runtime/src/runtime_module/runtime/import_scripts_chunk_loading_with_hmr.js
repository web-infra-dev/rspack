function loadUpdateChunk(chunkId, updatedModulesList) {
    var success = false;
    $globalObject$[$hotUpdateGlobal$] = function (_, moreModules, runtime) {
        for (var moduleId in moreModules) {
            if (__webpack_require__.o(moreModules, moduleId)) {
                currentUpdate[moduleId] = moreModules[moduleId];
                if (updatedModulesList) updatedModulesList.push(moduleId);
            }
        }
        if (runtime) currentUpdateRuntime.push(runtime);
        success = true;
    };
    // start update chunk loading
    importScripts($URL$);
    if (!success) throw new Error("Loading update chunk failed for unknown reason");
}
