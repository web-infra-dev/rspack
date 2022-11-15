(function() {// mount Modules
(function () {
	runtime.installedModules = {
"./answer.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "answer", {
    enumerable: true,
    get: ()=>answer
});
const answer = 103330;
},
"./app.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _lib = __rspack_require__("./lib.js");
setTimeout(()=>{
    console.log(_lib.myanswer);
}, 1000);
if (module.hot?.accept) module.hot.accept((module1)=>{
    console.log("xxx:", module1);
});
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
},
"./lib.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "myanswer", {
    enumerable: true,
    get: ()=>myanswer
});
const _answer = __rspack_require__("./answer.js");
const myanswer = _answer.answer;
},

};
})();

// mount Chunks
(function () {
	runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
	runtime.moduleCache = {};
})();
self["__rspack_runtime__"].__rspack_require__("./index.js");})()