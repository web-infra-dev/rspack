(function() {
var __webpack_modules__ = {

}
// The module cache
 var __webpack_module_cache__ = {};
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
// webpack/runtime/ensure_chunk
(function() {
__webpack_require__.f = {};
// This file contains only the entry chunk.
// The chunk loading function for additional chunks
__webpack_require__.e = function (chunkId) {
	return Promise.all(
		Object.keys(__webpack_require__.f).reduce(function (promises, key) {
			__webpack_require__.f[key](chunkId, promises);
			return promises;
		}, [])
	);
};

})();
// ir
(function() {
function _getRequireCache(nodeInterop) {
	if (typeof WeakMap !== "function") return null;

	var cacheBabelInterop = new WeakMap();
	var cacheNodeInterop = new WeakMap();
	return (_getRequireCache = function (nodeInterop) {
		return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
	})(nodeInterop);
}

__webpack_require__.ir = function (obj, nodeInterop) {
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
// webpack/runtime/load_chunk_with_module
(function() {
var map = {"./child/a.js": ["child_a_js",],"./child/b.js": ["child_b_js",],};

    __webpack_require__.el = function(module) {
        var chunkId = map[module];
        if (chunkId === undefined) {
            return Promise.resolve();
        }
        if (chunkId.length > 1) {
          return Promise.all(chunkId.map(__webpack_require__.e));
        } else {
          return __webpack_require__.e(chunkId[0]);
        };
    }
    
})();
// webpack/runtime/has_own_property
(function() {
__webpack_require__.o = function (obj, prop) {
	return Object.prototype.hasOwnProperty.call(obj, prop);
};

})();
// webpack/runtime/get_chunk_filename/__webpack_require__.u
(function() {
// This function allow to reference chunks
        __webpack_require__.u = function (chunkId) {
          // return url for filenames based on template
          return {"child_a_js": "child_a_js.js","child_b_js": "child_b_js.js",}[chunkId];
        };
      
})();
// webpack/runtime/require_chunk_loading
(function() {
var installedChunks = {"runtime": 0,};
// object to store loaded chunks
// "1" means "loaded", otherwise not loaded yet

var installChunk = function (chunk) {
	var moreModules = chunk.modules,
		chunkIds = chunk.ids,
		runtime = chunk.runtime;
	for (var moduleId in moreModules) {
		if (__webpack_require__.o(moreModules, moduleId)) {
			__webpack_require__.m[moduleId] = moreModules[moduleId];
		}
	}
	if (runtime) runtime(__webpack_require__);
	for (var i = 0; i < chunkIds.length; i++) installedChunks[chunkIds[i]] = 1;
	
};
// require() chunk loading for javascript
__webpack_require__.f.require = function (chunkId, promises) {
	// "1" is the signal for "already loaded"
	if (!installedChunks[chunkId]) {
		if (chunkId) {
			installChunk(require("./" + __webpack_require__.u(chunkId)));
		} else installedChunks[chunkId] = 1;
	}
};
module.exports = __webpack_require__;
__webpack_require__.C = installChunk;

})();

})()
