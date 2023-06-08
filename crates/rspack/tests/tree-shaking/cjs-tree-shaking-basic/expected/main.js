(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./answer.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'answer': function() { return answer; }});

 const answer = 42;
},
"./app.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);

},
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);

__webpack_require__(/* ./answer */"./answer.js");
myanswer();
},
"./lib.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'myanswer': function() { return myanswer; }});
 const myanswer = 'anyser';
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);