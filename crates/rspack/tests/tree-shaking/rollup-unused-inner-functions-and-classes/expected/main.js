(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _stuffJs = __webpack_require__("./stuff.js");
(0, _stuffJs.bar)();
var f = (0, _stuffJs.baz)();
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
Object.defineProperty(exports, "baz", {
    enumerable: true,
    get: ()=>Baz
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
__webpack_require__.d(exports, {
    "bar": ()=>bar,
    "baz": ()=>Baz
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);