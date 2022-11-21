self["__rspack_runtime__"].__rspack_register__(["default"], 
{
"./node_modules/foo/index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
console.log('foo');
const _default = 'foo';
},
"./shared.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
console.log('shared.js');
},

});