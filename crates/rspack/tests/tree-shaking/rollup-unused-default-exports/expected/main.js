(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
var foo = {
    value: 1
};
function mutate(obj) {
    obj.value += 1;
    return obj;
}
__webpack_require__.d(exports, {
    "foo": ()=>foo
});
mutate(foo);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _fooJs = __webpack_require__("./foo.js");
assert.equal(_fooJs.foo.value, 2);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);