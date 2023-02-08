(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./import-module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _moduleJs = __webpack_require__.ir(__webpack_require__("./module.js"));
__webpack_require__.d(exports, {
    "test": ()=>test
});
function test() {
    (0, _moduleJs.default)({
        type: "inline"
    });
}
},
"./index.js": function (module, exports, __webpack_require__) {
it("should generate correct code when pure expressions are in dead branches", ()=>{
    __webpack_require__("./import-module.js").test();
    return Promise.all([
        Promise.all([]).then(__webpack_require__.bind(__webpack_require__, "./some-module.js")).then(__webpack_require__.ir),
        __webpack_require__.e("chunk_js").then(__webpack_require__.bind(__webpack_require__, "./chunk.js")).then(__webpack_require__.ir)
    ]);
});
},
"./module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _someModuleJs = __webpack_require__("./some-module.js");
function getType(obj) {
    return obj.type;
}
function doSomethingWithBlock(obj) {
    return _someModuleJs.Block.doSomething(obj);
}
function doSomethingWithInline(obj) {
    return _someModuleJs.Inline.doSomething(obj);
}
function doSomethingWithDocument(obj) {
    return _someModuleJs.Document.doSomething(obj);
}
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
__webpack_require__.d(exports, {
    "default": ()=>__RSPACK_DEFAULT_EXPORT__
});
let __RSPACK_DEFAULT_EXPORT__ = doSomething;
},
"./some-module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    Block: ()=>Block,
    Inline: ()=>Inline,
    Document: ()=>Document
});
class Block {
    static doSomething() {}
}
class Inline {
    static doSomething() {}
}
class Document {
    static doSomething() {}
}
__webpack_require__.d(exports, {
    "Inline": ()=>Inline,
    "Block": ()=>Block,
    "Document": ()=>Document
});
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);