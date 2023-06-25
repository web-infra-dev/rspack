(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./import-module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'test': function() { return test; }});
/* harmony import */var _module__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./module */"./module.js");

 function test() {
    (0, _module__WEBPACK_IMPORTED_MODULE__["default"])({
        type: "inline"
    });
}
},
"./index.js": function (module, exports, __webpack_require__) {
it("should generate correct code when pure expressions are in dead branches", ()=>{
    __webpack_require__(/* ./import-module */"./import-module.js").test();
    return Promise.all([
        __webpack_require__.el(/* ./some-module */"./some-module.js").then(__webpack_require__.bind(__webpack_require__, /* ./some-module */"./some-module.js")),
        __webpack_require__.el(/* ./chunk */"./chunk.js").then(__webpack_require__.bind(__webpack_require__, /* ./chunk */"./chunk.js"))
    ]);
});
},
"./module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'default': function() { return __WEBPACK_DEFAULT_EXPORT__; }});
/* harmony import */var _some_module__WEBPACK_IMPORTED_MODULE__ = __webpack_require__(/* ./some-module */"./some-module.js");

function getType(obj) {
    return obj.type;
}
// Local functions
function doSomethingWithBlock(obj) {
    return (0, _some_module__WEBPACK_IMPORTED_MODULE__["Block"]).doSomething(obj);
}
function doSomethingWithInline(obj) {
    return (0, _some_module__WEBPACK_IMPORTED_MODULE__["Inline"]).doSomething(obj);
}
function doSomethingWithDocument(obj) {
    return (0, _some_module__WEBPACK_IMPORTED_MODULE__["Document"]).doSomething(obj);
}
// Exported functions
function doSomething(obj) {
    const type = getType(obj);
    switch(type){
        case "document":
            return doSomethingWithDocument(obj);
        case "block":
            return doSomethingWithBlock(obj);
        case "inline":
            return doSomethingWithInline(obj);
        default:
            throw new Error();
    }
}
function useDocument(obj) {
    return doSomethingWithDocument(obj);
}

var __WEBPACK_DEFAULT_EXPORT__ = doSomething;
},
"./some-module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'Block': function() { return Block; }, 'Inline': function() { return Inline; }, 'Document': function() { return Document; }});
class Block {
    static doSomething() {}
}
class Inline {
    static doSomething() {}
}
class Document {
    static doSomething() {}
}

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);