(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./Layout.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.exportStar(__webpack_require__("./something/index.js"), exports);
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
__webpack_require__.exportStar(__webpack_require__("./colors/result.js"), exports);
},
"./colors/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.exportStar(__webpack_require__("./colors/a.js"), exports);
__webpack_require__.exportStar(__webpack_require__("./colors/b.js"), exports);
__webpack_require__.exportStar(__webpack_require__("./colors/c.js"), exports);
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
<<<<<<< HEAD
Object.defineProperty(exports, "Something", {
    enumerable: true,
    get: ()=>_layoutJs.Something
});
const _layoutJs = __webpack_require__("./Layout.js");
=======
__webpack_require__.exportStar(__webpack_require__("./Layout.js"), exports);
>>>>>>> 6da20c06 (fix: 🐛 fix tree-shaking issue)
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
<<<<<<< HEAD
const _exportJs = __webpack_require__("./export.js");
_exportJs.Something;
=======
const _export = __webpack_require__("./export.js");
_export.Colors.result;
_export.Something;
>>>>>>> 6da20c06 (fix: 🐛 fix tree-shaking issue)
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
    get: ()=>_index
});
const _index = __webpack_require__.interopRequire(__webpack_require__("./colors/index.js"));
__webpack_require__.exportStar(__webpack_require__("./something/Something.js"), exports);
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);