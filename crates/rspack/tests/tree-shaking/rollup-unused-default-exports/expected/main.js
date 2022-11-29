(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./foo.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "foo", {
    enumerable: true,
    get: ()=>foo
});
var foo = {
    value: 1
};
function mutate(obj) {
    obj.value += 1;
    return obj;
}
mutate(foo);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _foo = __webpack_require__("./foo.js");
assert.equal(_foo.foo.value, 2);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);