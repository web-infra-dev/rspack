(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
/* harmony import */var _stuff__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./stuff */"./stuff.js");

(0, _stuff__WEBPACK_IMPORTED_MODULE__["bar"])();
var f = (0, _stuff__WEBPACK_IMPORTED_MODULE__["baz"])();
f();
function getClass() {
    class MyClass {
    }
    class UnusedInnerClass1 {
    }
    return MyClass;
}
class UnusedClass {
}
console.log(getClass().name);
},
"./stuff.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'bar': function() { return bar; }, 'baz': function() { return Baz; }});
 function foo() {
    console.log("outer foo");
}
 function bar() {
    console.log("outer bar");
}
 function bog() {
    console.log("outer bog");
}
 function boo() {
    console.log("outer boo");
}
function Baz() {
    function foo() {
        console.log("inner foo");
    }
    function bar() {
        console.log("inner bar");
    }
    function bog() {
        console.log("inner bog");
    }
    function boo() {
        console.log("inner boo");
    }
    return bar(), bog;
}

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);