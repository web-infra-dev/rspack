(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./import-module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "test", {
    enumerable: true,
    get: ()=>test
});
const _module = __webpack_require__.interopRequire(__webpack_require__("./module.js"));
function test() {
    (0, _module.default)({
        type: "inline"
    });
}
},
"./index.js": function (module, exports, __webpack_require__) {
it("should generate correct code when pure expressions are in dead branches", ()=>{
    __webpack_require__("./import-module.js").test();
    return Promise.all([
        __webpack_require__.e("some-module_js").then(__webpack_require__.bind(__webpack_require__, "./some-module.js")).then(__webpack_require__.interopRequire),
        __webpack_require__.e("chunk_js").then(__webpack_require__.bind(__webpack_require__, "./chunk.js")).then(__webpack_require__.interopRequire)
    ]);
});
},
"./module.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: ()=>_default
});
const _someModule = __webpack_require__("./some-module.js");
function getType(obj) {
    return obj.type;
}
function doSomethingWithBlock(obj) {
    return _someModule.Block.doSomething(obj);
}
function doSomethingWithInline(obj) {
    return _someModule.Inline.doSomething(obj);
}
function doSomethingWithDocument(obj) {
    return _someModule.Document.doSomething(obj);
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
const _default = doSomething;
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
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);