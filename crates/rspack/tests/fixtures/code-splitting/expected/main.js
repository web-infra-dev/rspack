self["__rspack_runtime__"].__rspack_register__(["main"], {
"./index.js": function (module, exports, __rspack_require__, __rspack_dynamic_require__) {
"use strict";
console.log('hello, world');
__rspack_dynamic_require__([
    "a_js"
]).then(__rspack_require__.bind(__rspack_require__, "./a.js"));
__rspack_dynamic_require__([
    "b_js"
]).then(__rspack_require__.bind(__rspack_require__, "./b.js"));
},
});self["__rspack_runtime__"].__rspack_require__("./index.js");