(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["async-entry_js"], {
"./async-entry.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__("./shared.js");
__webpack_require__("./node_modules/foo/index.js");
},
"./node_modules/foo/index.js": function (module, exports, __webpack_require__) {
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
"./shared.js": function (module, exports, __webpack_require__) {
"use strict";
console.log('shared.js');
},

}]);