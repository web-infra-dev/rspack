(function() {
// var __webpack_modules__ = ({});
// replace here to modules
var __webpack_modules__ = {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _stuff = __webpack_require__("./stuff.js");
(0, _stuff.bar)();
var f = (0, _stuff.baz)();
f();
function getClass() {
    class MyClass {
    }
    return MyClass;
}
console.log(getClass().name);
},
"./stuff.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    bar: ()=>bar,
    baz: ()=>Baz
});
function bar() {
    console.log("outer bar");
}
function Baz() {
    function bar() {
        console.log("inner bar");
    }
    function bog() {
        console.log("inner bog");
    }
    return bar(), bog;
}
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
__webpack_require__("./index.js");
})();
