self["__rspack_runtime__"].__rspack_register__(["main"], {
"./a.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__rspack_require__("./b.js");
console.log('a');
},
"./b.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
console.log('b');
},
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__, __rspack_runtime__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__rspack_require__("./a.js");
console.log('hello, world');
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");