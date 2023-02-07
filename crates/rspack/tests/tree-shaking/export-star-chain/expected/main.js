(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./something/index.js"), exports);
},
"./colors/a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "red", {
    enumerable: true,
    get: ()=>red
});
const red = 'red';
},
"./colors/b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "blue", {
    enumerable: true,
    get: ()=>blue
});
const blue = 'blue';
},
"./colors/c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./colors/result.js"), exports);
},
"./colors/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./colors/a.js"), exports);
__webpack_require__.es(__webpack_require__("./colors/b.js"), exports);
__webpack_require__.es(__webpack_require__("./colors/c.js"), exports);
},
"./colors/result.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "result", {
    enumerable: true,
    get: ()=>result
});
const result = 'ssss';
},
"./export.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./Layout.js"), exports);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _exportJs = __webpack_require__("./export.js");
_exportJs.Colors;
_exportJs.Something;
},
"./something/Something.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Something", {
    enumerable: true,
    get: ()=>Something
});
class Something {
}
},
"./something/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "Colors", {
    enumerable: true,
    get: ()=>_indexJs
});
const _indexJs = __webpack_require__.ir(__webpack_require__("./colors/index.js"));
__webpack_require__.es(__webpack_require__("./something/Something.js"), exports);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);