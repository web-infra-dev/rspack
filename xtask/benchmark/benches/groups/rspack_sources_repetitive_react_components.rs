#![allow(missing_docs)]
#![allow(clippy::unreadable_literal)]

use std::sync::LazyLock;

use criterion::Bencher;
use rspack_sources::{
  BoxSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawStringSource, ReplaceSource,
  ReplacementEnforce, Source, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
};

static REPETITIVE_1K_REACT_COMPONENTS_SOURCE: LazyLock<BoxSource> = LazyLock::new(|| {
  ConcatSource::new(vec![
      RawStringSource::from_static("(() => { // webpackBootstrap\n").boxed(),
      RawStringSource::from_static("\"use strict\";\n").boxed(),
      RawStringSource::from_static("var __webpack_modules__ = (").boxed(),
      RawStringSource::from_static("{\n").boxed(),
      RawStringSource::from_static("\"./index.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*******************!*\\\n  !*** ./index.jsx ***!\n  \\*******************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_dom_client__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! react-dom/client */ \"../../node_modules/.pnpm/react-dom@18.2.0_react@18.2.0/node_modules/react-dom/client.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _src_f0__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./src/f0 */ \"./src/f0.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from \"react\";\nimport ReactDom from \"react-dom/client\";\nimport App1 from \"./src/f0\";\nReactDom.createRoot(document.getElementById(\"root\")).render(/*#__PURE__*/ _jsx(React.StrictMode, {\n    children: /*#__PURE__*/ _jsx(App1, {})\n}));\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/index.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/index.jsx\"],\"sourcesContent\":[\"\\nimport React, { useEffect } from \\\"react\\\";\\nimport ReactDom from \\\"react-dom/client\\\";\\nimport App1 from \\\"./src/f0\\\";\\n\\nReactDom.createRoot(document.getElementById(\\\"root\\\")).render(\\n\\t<React.StrictMode>\\n\\t\\t<App1 />\\n\\t</React.StrictMode>\\n);\\n\\n\\t\\t\\n\\t\\t\"],\"names\":[\"React\",\"ReactDom\",\"App1\",\"document\"],\"mappings\":\";AACA,OAAOA,WAA0B,QAAQ;AACzC,OAAOC,cAAc,mBAAmB;AACxC,OAAOC,UAAU,WAAW;AAE5BD,SAAS,UAAU,CAACE,SAAS,cAAc,CAAC,SAAS,MAAM,eAC1D,KAACH,MAAM,UAAU;cAChB,mBAACE\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(76, 116, "", None);
        source.replace_static(117, 145, "", None);
        source.replace_static(146, 165, "react_dom_client__WEBPACK_IMPORTED_MODULE_2__.createRoot", None);
        source.replace_static(220, 224, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(225, 241, "(react__WEBPACK_IMPORTED_MODULE_1___default().StrictMode)", None);
        source.replace_static(273, 277, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(278, 282, "_src_f0__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React, { useEffect } from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    useEffect(()=>{\n        console.log(Date.now());\n    });\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(\"span\", {\n                children: \"    19  \"\n            }),\n            Date.now()\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f0.jsx\"],\"sourcesContent\":[\"\\n\\nimport React, {useEffect} from 'react'\\nimport Icon  from '@icon-park/react/es/all';\\n\\n\\nfunction Navbar({ show }) {\\nuseEffect(() => {\\n  console.log(Date.now());\\n})\\nreturn (\\n  <div>\\n  <span>    19  </span>\\n  {Date.now()}\\n  </div>\\n)\\n}\\n\\nexport default Navbar\\n\\n\\n\"],\"names\":[\"React\",\"useEffect\",\"Navbar\",\"param\",\"show\",\"console\",\"Date\"],\"mappings\":\";AAEA,OAAOA,SAAQC,SAAS,QAAO,QAAO;AAItC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAClBH,UAAU;QACRI,QAAQ,GAAG,CAACC,KAAK,GAAG;IACtB;IACA,qBACE,MAAC;;0BACD,KAAC;0BAAK;;YACLA,KAAK,GAAG;;;AAGX;AAEA,eAAeJ,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 105, "", None);
        source.replace_static(162, 171, "(0,react__WEBPACK_IMPORTED_MODULE_1__.useEffect)", None);
        source.replace_static(244, 249, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(305, 309, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(416, 431, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(416, 431, "", None);
        source.replace_static_with_enforce(437, 438, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d0/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d0/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d0/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d1/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d1/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d1/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d2/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d2/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d2/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d3/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d3/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d3/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d4/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d4/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d4/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d5/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d5/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d5/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d6/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d6/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d6/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d7/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d7/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d7/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d8/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d8/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d8/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f0.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f1.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f2.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f3.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f4.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f5.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f6.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f7.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/d9/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************************!*\\\n  !*** ./src/d0/d0/d0/d9/f8.jsx ***!\n  \\********************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx } from \"react/jsx-runtime\";\nimport React from 'react';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsx(\"div\", {});\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/d9/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  \\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      \\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAIzB,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,KAAC;AAIL;AAEA,eAAeF,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 48, "", None);
        source.replace_static(49, 75, "", None);
        source.replace_static(153, 157, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(172, 187, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(172, 187, "", None);
        source.replace_static_with_enforce(193, 194, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f0.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d0/f0.jsx */ \"./src/d0/d0/d0/d0/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d0/f1.jsx */ \"./src/d0/d0/d0/d0/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d0/f2.jsx */ \"./src/d0/d0/d0/d0/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d0/f3.jsx */ \"./src/d0/d0/d0/d0/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d0/f4.jsx */ \"./src/d0/d0/d0/d0/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d0/f5.jsx */ \"./src/d0/d0/d0/d0/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d0/f6.jsx */ \"./src/d0/d0/d0/d0/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d0/f7.jsx */ \"./src/d0/d0/d0/d0/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d0/f8.jsx */ \"./src/d0/d0/d0/d0/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d0/f0.jsx';\nimport Component__1 from './d0/f1.jsx';\nimport Component__2 from './d0/f2.jsx';\nimport Component__3 from './d0/f3.jsx';\nimport Component__4 from './d0/f4.jsx';\nimport Component__5 from './d0/f5.jsx';\nimport Component__6 from './d0/f6.jsx';\nimport Component__7 from './d0/f7.jsx';\nimport Component__8 from './d0/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d0/f0.jsx'\\nimport Component__1 from './d0/f1.jsx'\\nimport Component__2 from './d0/f2.jsx'\\nimport Component__3 from './d0/f3.jsx'\\nimport Component__4 from './d0/f4.jsx'\\nimport Component__5 from './d0/f5.jsx'\\nimport Component__6 from './d0/f6.jsx'\\nimport Component__7 from './d0/f7.jsx'\\nimport Component__8 from './d0/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d0_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d0_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d0_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d0_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d0_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d0_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d0_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d0_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f1.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f1.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d1/f0.jsx */ \"./src/d0/d0/d0/d1/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d1/f1.jsx */ \"./src/d0/d0/d0/d1/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d1/f2.jsx */ \"./src/d0/d0/d0/d1/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d1/f3.jsx */ \"./src/d0/d0/d0/d1/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d1/f4.jsx */ \"./src/d0/d0/d0/d1/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d1/f5.jsx */ \"./src/d0/d0/d0/d1/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d1/f6.jsx */ \"./src/d0/d0/d0/d1/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d1/f7.jsx */ \"./src/d0/d0/d0/d1/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d1_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d1/f8.jsx */ \"./src/d0/d0/d0/d1/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d1/f0.jsx';\nimport Component__1 from './d1/f1.jsx';\nimport Component__2 from './d1/f2.jsx';\nimport Component__3 from './d1/f3.jsx';\nimport Component__4 from './d1/f4.jsx';\nimport Component__5 from './d1/f5.jsx';\nimport Component__6 from './d1/f6.jsx';\nimport Component__7 from './d1/f7.jsx';\nimport Component__8 from './d1/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f1.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f1.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d1/f0.jsx'\\nimport Component__1 from './d1/f1.jsx'\\nimport Component__2 from './d1/f2.jsx'\\nimport Component__3 from './d1/f3.jsx'\\nimport Component__4 from './d1/f4.jsx'\\nimport Component__5 from './d1/f5.jsx'\\nimport Component__6 from './d1/f6.jsx'\\nimport Component__7 from './d1/f7.jsx'\\nimport Component__8 from './d1/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d1_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d1_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d1_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d1_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d1_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d1_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d1_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d1_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d1_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f2.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f2.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d2/f0.jsx */ \"./src/d0/d0/d0/d2/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d2/f1.jsx */ \"./src/d0/d0/d0/d2/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d2/f2.jsx */ \"./src/d0/d0/d0/d2/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d2/f3.jsx */ \"./src/d0/d0/d0/d2/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d2/f4.jsx */ \"./src/d0/d0/d0/d2/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d2/f5.jsx */ \"./src/d0/d0/d0/d2/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d2/f6.jsx */ \"./src/d0/d0/d0/d2/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d2/f7.jsx */ \"./src/d0/d0/d0/d2/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d2_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d2/f8.jsx */ \"./src/d0/d0/d0/d2/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d2/f0.jsx';\nimport Component__1 from './d2/f1.jsx';\nimport Component__2 from './d2/f2.jsx';\nimport Component__3 from './d2/f3.jsx';\nimport Component__4 from './d2/f4.jsx';\nimport Component__5 from './d2/f5.jsx';\nimport Component__6 from './d2/f6.jsx';\nimport Component__7 from './d2/f7.jsx';\nimport Component__8 from './d2/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f2.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f2.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d2/f0.jsx'\\nimport Component__1 from './d2/f1.jsx'\\nimport Component__2 from './d2/f2.jsx'\\nimport Component__3 from './d2/f3.jsx'\\nimport Component__4 from './d2/f4.jsx'\\nimport Component__5 from './d2/f5.jsx'\\nimport Component__6 from './d2/f6.jsx'\\nimport Component__7 from './d2/f7.jsx'\\nimport Component__8 from './d2/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d2_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d2_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d2_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d2_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d2_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d2_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d2_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d2_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d2_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f3.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f3.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d3/f0.jsx */ \"./src/d0/d0/d0/d3/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d3/f1.jsx */ \"./src/d0/d0/d0/d3/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d3/f2.jsx */ \"./src/d0/d0/d0/d3/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d3/f3.jsx */ \"./src/d0/d0/d0/d3/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d3/f4.jsx */ \"./src/d0/d0/d0/d3/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d3/f5.jsx */ \"./src/d0/d0/d0/d3/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d3/f6.jsx */ \"./src/d0/d0/d0/d3/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d3/f7.jsx */ \"./src/d0/d0/d0/d3/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d3_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d3/f8.jsx */ \"./src/d0/d0/d0/d3/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d3/f0.jsx';\nimport Component__1 from './d3/f1.jsx';\nimport Component__2 from './d3/f2.jsx';\nimport Component__3 from './d3/f3.jsx';\nimport Component__4 from './d3/f4.jsx';\nimport Component__5 from './d3/f5.jsx';\nimport Component__6 from './d3/f6.jsx';\nimport Component__7 from './d3/f7.jsx';\nimport Component__8 from './d3/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f3.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f3.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d3/f0.jsx'\\nimport Component__1 from './d3/f1.jsx'\\nimport Component__2 from './d3/f2.jsx'\\nimport Component__3 from './d3/f3.jsx'\\nimport Component__4 from './d3/f4.jsx'\\nimport Component__5 from './d3/f5.jsx'\\nimport Component__6 from './d3/f6.jsx'\\nimport Component__7 from './d3/f7.jsx'\\nimport Component__8 from './d3/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d3_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d3_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d3_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d3_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d3_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d3_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d3_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d3_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d3_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f4.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f4.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d4/f0.jsx */ \"./src/d0/d0/d0/d4/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d4/f1.jsx */ \"./src/d0/d0/d0/d4/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d4/f2.jsx */ \"./src/d0/d0/d0/d4/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d4/f3.jsx */ \"./src/d0/d0/d0/d4/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d4/f4.jsx */ \"./src/d0/d0/d0/d4/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d4/f5.jsx */ \"./src/d0/d0/d0/d4/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d4/f6.jsx */ \"./src/d0/d0/d0/d4/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d4/f7.jsx */ \"./src/d0/d0/d0/d4/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d4_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d4/f8.jsx */ \"./src/d0/d0/d0/d4/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d4/f0.jsx';\nimport Component__1 from './d4/f1.jsx';\nimport Component__2 from './d4/f2.jsx';\nimport Component__3 from './d4/f3.jsx';\nimport Component__4 from './d4/f4.jsx';\nimport Component__5 from './d4/f5.jsx';\nimport Component__6 from './d4/f6.jsx';\nimport Component__7 from './d4/f7.jsx';\nimport Component__8 from './d4/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f4.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f4.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d4/f0.jsx'\\nimport Component__1 from './d4/f1.jsx'\\nimport Component__2 from './d4/f2.jsx'\\nimport Component__3 from './d4/f3.jsx'\\nimport Component__4 from './d4/f4.jsx'\\nimport Component__5 from './d4/f5.jsx'\\nimport Component__6 from './d4/f6.jsx'\\nimport Component__7 from './d4/f7.jsx'\\nimport Component__8 from './d4/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d4_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d4_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d4_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d4_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d4_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d4_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d4_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d4_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d4_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f5.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f5.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d5/f0.jsx */ \"./src/d0/d0/d0/d5/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d5/f1.jsx */ \"./src/d0/d0/d0/d5/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d5/f2.jsx */ \"./src/d0/d0/d0/d5/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d5/f3.jsx */ \"./src/d0/d0/d0/d5/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d5/f4.jsx */ \"./src/d0/d0/d0/d5/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d5/f5.jsx */ \"./src/d0/d0/d0/d5/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d5/f6.jsx */ \"./src/d0/d0/d0/d5/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d5/f7.jsx */ \"./src/d0/d0/d0/d5/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d5_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d5/f8.jsx */ \"./src/d0/d0/d0/d5/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d5/f0.jsx';\nimport Component__1 from './d5/f1.jsx';\nimport Component__2 from './d5/f2.jsx';\nimport Component__3 from './d5/f3.jsx';\nimport Component__4 from './d5/f4.jsx';\nimport Component__5 from './d5/f5.jsx';\nimport Component__6 from './d5/f6.jsx';\nimport Component__7 from './d5/f7.jsx';\nimport Component__8 from './d5/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f5.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f5.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d5/f0.jsx'\\nimport Component__1 from './d5/f1.jsx'\\nimport Component__2 from './d5/f2.jsx'\\nimport Component__3 from './d5/f3.jsx'\\nimport Component__4 from './d5/f4.jsx'\\nimport Component__5 from './d5/f5.jsx'\\nimport Component__6 from './d5/f6.jsx'\\nimport Component__7 from './d5/f7.jsx'\\nimport Component__8 from './d5/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d5_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d5_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d5_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d5_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d5_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d5_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d5_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d5_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d5_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f6.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f6.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d6/f0.jsx */ \"./src/d0/d0/d0/d6/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d6/f1.jsx */ \"./src/d0/d0/d0/d6/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d6/f2.jsx */ \"./src/d0/d0/d0/d6/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d6/f3.jsx */ \"./src/d0/d0/d0/d6/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d6/f4.jsx */ \"./src/d0/d0/d0/d6/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d6/f5.jsx */ \"./src/d0/d0/d0/d6/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d6/f6.jsx */ \"./src/d0/d0/d0/d6/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d6/f7.jsx */ \"./src/d0/d0/d0/d6/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d6_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d6/f8.jsx */ \"./src/d0/d0/d0/d6/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d6/f0.jsx';\nimport Component__1 from './d6/f1.jsx';\nimport Component__2 from './d6/f2.jsx';\nimport Component__3 from './d6/f3.jsx';\nimport Component__4 from './d6/f4.jsx';\nimport Component__5 from './d6/f5.jsx';\nimport Component__6 from './d6/f6.jsx';\nimport Component__7 from './d6/f7.jsx';\nimport Component__8 from './d6/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f6.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f6.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d6/f0.jsx'\\nimport Component__1 from './d6/f1.jsx'\\nimport Component__2 from './d6/f2.jsx'\\nimport Component__3 from './d6/f3.jsx'\\nimport Component__4 from './d6/f4.jsx'\\nimport Component__5 from './d6/f5.jsx'\\nimport Component__6 from './d6/f6.jsx'\\nimport Component__7 from './d6/f7.jsx'\\nimport Component__8 from './d6/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d6_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d6_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d6_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d6_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d6_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d6_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d6_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d6_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d6_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f7.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f7.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d7/f0.jsx */ \"./src/d0/d0/d0/d7/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d7/f1.jsx */ \"./src/d0/d0/d0/d7/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d7/f2.jsx */ \"./src/d0/d0/d0/d7/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d7/f3.jsx */ \"./src/d0/d0/d0/d7/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d7/f4.jsx */ \"./src/d0/d0/d0/d7/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d7/f5.jsx */ \"./src/d0/d0/d0/d7/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d7/f6.jsx */ \"./src/d0/d0/d0/d7/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d7/f7.jsx */ \"./src/d0/d0/d0/d7/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d7_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d7/f8.jsx */ \"./src/d0/d0/d0/d7/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d7/f0.jsx';\nimport Component__1 from './d7/f1.jsx';\nimport Component__2 from './d7/f2.jsx';\nimport Component__3 from './d7/f3.jsx';\nimport Component__4 from './d7/f4.jsx';\nimport Component__5 from './d7/f5.jsx';\nimport Component__6 from './d7/f6.jsx';\nimport Component__7 from './d7/f7.jsx';\nimport Component__8 from './d7/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f7.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f7.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d7/f0.jsx'\\nimport Component__1 from './d7/f1.jsx'\\nimport Component__2 from './d7/f2.jsx'\\nimport Component__3 from './d7/f3.jsx'\\nimport Component__4 from './d7/f4.jsx'\\nimport Component__5 from './d7/f5.jsx'\\nimport Component__6 from './d7/f6.jsx'\\nimport Component__7 from './d7/f7.jsx'\\nimport Component__8 from './d7/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d7_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d7_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d7_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d7_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d7_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d7_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d7_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d7_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d7_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/d0/f8.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!*****************************!*\\\n  !*** ./src/d0/d0/d0/f8.jsx ***!\n  \\*****************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d8/f0.jsx */ \"./src/d0/d0/d0/d8/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d8/f1.jsx */ \"./src/d0/d0/d0/d8/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d8/f2.jsx */ \"./src/d0/d0/d0/d8/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d8/f3.jsx */ \"./src/d0/d0/d0/d8/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d8/f4.jsx */ \"./src/d0/d0/d0/d8/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d8/f5.jsx */ \"./src/d0/d0/d0/d8/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d8/f6.jsx */ \"./src/d0/d0/d0/d8/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d8/f7.jsx */ \"./src/d0/d0/d0/d8/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d8_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d8/f8.jsx */ \"./src/d0/d0/d0/d8/f8.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f0_jsx__WEBPACK_IMPORTED_MODULE_11__ = __webpack_require__(/*! ./d9/f0.jsx */ \"./src/d0/d0/d0/d9/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f1_jsx__WEBPACK_IMPORTED_MODULE_12__ = __webpack_require__(/*! ./d9/f1.jsx */ \"./src/d0/d0/d0/d9/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f2_jsx__WEBPACK_IMPORTED_MODULE_13__ = __webpack_require__(/*! ./d9/f2.jsx */ \"./src/d0/d0/d0/d9/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f3_jsx__WEBPACK_IMPORTED_MODULE_14__ = __webpack_require__(/*! ./d9/f3.jsx */ \"./src/d0/d0/d0/d9/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f4_jsx__WEBPACK_IMPORTED_MODULE_15__ = __webpack_require__(/*! ./d9/f4.jsx */ \"./src/d0/d0/d0/d9/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f5_jsx__WEBPACK_IMPORTED_MODULE_16__ = __webpack_require__(/*! ./d9/f5.jsx */ \"./src/d0/d0/d0/d9/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f6_jsx__WEBPACK_IMPORTED_MODULE_17__ = __webpack_require__(/*! ./d9/f6.jsx */ \"./src/d0/d0/d0/d9/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f7_jsx__WEBPACK_IMPORTED_MODULE_18__ = __webpack_require__(/*! ./d9/f7.jsx */ \"./src/d0/d0/d0/d9/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d9_f8_jsx__WEBPACK_IMPORTED_MODULE_19__ = __webpack_require__(/*! ./d9/f8.jsx */ \"./src/d0/d0/d0/d9/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d8/f0.jsx';\nimport Component__1 from './d8/f1.jsx';\nimport Component__2 from './d8/f2.jsx';\nimport Component__3 from './d8/f3.jsx';\nimport Component__4 from './d8/f4.jsx';\nimport Component__5 from './d8/f5.jsx';\nimport Component__6 from './d8/f6.jsx';\nimport Component__7 from './d8/f7.jsx';\nimport Component__8 from './d8/f8.jsx';\nimport Component__9 from './d9/f0.jsx';\nimport Component__10 from './d9/f1.jsx';\nimport Component__11 from './d9/f2.jsx';\nimport Component__12 from './d9/f3.jsx';\nimport Component__13 from './d9/f4.jsx';\nimport Component__14 from './d9/f5.jsx';\nimport Component__15 from './d9/f6.jsx';\nimport Component__16 from './d9/f7.jsx';\nimport Component__17 from './d9/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {}),\n            /*#__PURE__*/ _jsx(Component__9, {}),\n            /*#__PURE__*/ _jsx(Component__10, {}),\n            /*#__PURE__*/ _jsx(Component__11, {}),\n            /*#__PURE__*/ _jsx(Component__12, {}),\n            /*#__PURE__*/ _jsx(Component__13, {}),\n            /*#__PURE__*/ _jsx(Component__14, {}),\n            /*#__PURE__*/ _jsx(Component__15, {}),\n            /*#__PURE__*/ _jsx(Component__16, {}),\n            /*#__PURE__*/ _jsx(Component__17, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f8.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/d0/f8.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d8/f0.jsx'\\nimport Component__1 from './d8/f1.jsx'\\nimport Component__2 from './d8/f2.jsx'\\nimport Component__3 from './d8/f3.jsx'\\nimport Component__4 from './d8/f4.jsx'\\nimport Component__5 from './d8/f5.jsx'\\nimport Component__6 from './d8/f6.jsx'\\nimport Component__7 from './d8/f7.jsx'\\nimport Component__8 from './d8/f8.jsx'\\nimport Component__9 from './d9/f0.jsx'\\nimport Component__10 from './d9/f1.jsx'\\nimport Component__11 from './d9/f2.jsx'\\nimport Component__12 from './d9/f3.jsx'\\nimport Component__13 from './d9/f4.jsx'\\nimport Component__14 from './d9/f5.jsx'\\nimport Component__15 from './d9/f6.jsx'\\nimport Component__16 from './d9/f7.jsx'\\nimport Component__17 from './d9/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n<Component__9/>\\n<Component__10/>\\n<Component__11/>\\n<Component__12/>\\n<Component__13/>\\n<Component__14/>\\n<Component__15/>\\n<Component__16/>\\n<Component__17/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Component__9\",\"Component__10\",\"Component__11\",\"Component__12\",\"Component__13\",\"Component__14\",\"Component__15\",\"Component__16\",\"Component__17\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACvC,OAAOC,mBAAmB,cAAa;AACrC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACpB;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(451, 490, "", None);
        source.replace_static(491, 531, "", None);
        source.replace_static(532, 572, "", None);
        source.replace_static(573, 613, "", None);
        source.replace_static(614, 654, "", None);
        source.replace_static(655, 695, "", None);
        source.replace_static(696, 736, "", None);
        source.replace_static(737, 777, "", None);
        source.replace_static(778, 818, "", None);
        source.replace_static(896, 901, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(957, 961, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(962, 974, "_d8_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(1007, 1011, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1012, 1024, "_d8_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(1057, 1061, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1062, 1074, "_d8_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(1107, 1111, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1112, 1124, "_d8_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(1157, 1161, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1162, 1174, "_d8_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(1207, 1211, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1212, 1224, "_d8_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(1257, 1261, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1262, 1274, "_d8_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(1307, 1311, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1312, 1324, "_d8_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(1357, 1361, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1362, 1374, "_d8_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1407, 1411, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1412, 1424, "_d9_f0_jsx__WEBPACK_IMPORTED_MODULE_11__[\"default\"]", None);
        source.replace_static(1457, 1461, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1462, 1475, "_d9_f1_jsx__WEBPACK_IMPORTED_MODULE_12__[\"default\"]", None);
        source.replace_static(1508, 1512, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1513, 1526, "_d9_f2_jsx__WEBPACK_IMPORTED_MODULE_13__[\"default\"]", None);
        source.replace_static(1559, 1563, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1564, 1577, "_d9_f3_jsx__WEBPACK_IMPORTED_MODULE_14__[\"default\"]", None);
        source.replace_static(1610, 1614, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1615, 1628, "_d9_f4_jsx__WEBPACK_IMPORTED_MODULE_15__[\"default\"]", None);
        source.replace_static(1661, 1665, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1666, 1679, "_d9_f5_jsx__WEBPACK_IMPORTED_MODULE_16__[\"default\"]", None);
        source.replace_static(1712, 1716, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1717, 1730, "_d9_f6_jsx__WEBPACK_IMPORTED_MODULE_17__[\"default\"]", None);
        source.replace_static(1763, 1767, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1768, 1781, "_d9_f7_jsx__WEBPACK_IMPORTED_MODULE_18__[\"default\"]", None);
        source.replace_static(1814, 1818, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1819, 1832, "_d9_f8_jsx__WEBPACK_IMPORTED_MODULE_19__[\"default\"]", None);
        source.replace_static(1858, 1873, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1858, 1873, "", None);
        source.replace_static_with_enforce(1879, 1880, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/d0/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!**************************!*\\\n  !*** ./src/d0/d0/f0.jsx ***!\n  \\**************************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d0/f0.jsx */ \"./src/d0/d0/d0/f0.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f1_jsx__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./d0/f1.jsx */ \"./src/d0/d0/d0/f1.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f2_jsx__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./d0/f2.jsx */ \"./src/d0/d0/d0/f2.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f3_jsx__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./d0/f3.jsx */ \"./src/d0/d0/d0/f3.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f4_jsx__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./d0/f4.jsx */ \"./src/d0/d0/d0/f4.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f5_jsx__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./d0/f5.jsx */ \"./src/d0/d0/d0/f5.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f6_jsx__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./d0/f6.jsx */ \"./src/d0/d0/d0/f6.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f7_jsx__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./d0/f7.jsx */ \"./src/d0/d0/d0/f7.jsx\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f8_jsx__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./d0/f8.jsx */ \"./src/d0/d0/d0/f8.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d0/f0.jsx';\nimport Component__1 from './d0/f1.jsx';\nimport Component__2 from './d0/f2.jsx';\nimport Component__3 from './d0/f3.jsx';\nimport Component__4 from './d0/f4.jsx';\nimport Component__5 from './d0/f5.jsx';\nimport Component__6 from './d0/f6.jsx';\nimport Component__7 from './d0/f7.jsx';\nimport Component__8 from './d0/f8.jsx';\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/d0/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d0/f0.jsx'\\nimport Component__1 from './d0/f1.jsx'\\nimport Component__2 from './d0/f2.jsx'\\nimport Component__3 from './d0/f3.jsx'\\nimport Component__4 from './d0/f4.jsx'\\nimport Component__5 from './d0/f5.jsx'\\nimport Component__6 from './d0/f6.jsx'\\nimport Component__7 from './d0/f7.jsx'\\nimport Component__8 from './d0/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACtC,OAAOC,kBAAkB,cAAa;AACpC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACX;0BACP,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(131, 170, "", None);
        source.replace_static(171, 210, "", None);
        source.replace_static(211, 250, "", None);
        source.replace_static(251, 290, "", None);
        source.replace_static(291, 330, "", None);
        source.replace_static(331, 370, "", None);
        source.replace_static(371, 410, "", None);
        source.replace_static(411, 450, "", None);
        source.replace_static(528, 533, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(589, 593, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(594, 606, "_d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(639, 643, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(644, 656, "_d0_f1_jsx__WEBPACK_IMPORTED_MODULE_3__[\"default\"]", None);
        source.replace_static(689, 693, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(694, 706, "_d0_f2_jsx__WEBPACK_IMPORTED_MODULE_4__[\"default\"]", None);
        source.replace_static(739, 743, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(744, 756, "_d0_f3_jsx__WEBPACK_IMPORTED_MODULE_5__[\"default\"]", None);
        source.replace_static(789, 793, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(794, 806, "_d0_f4_jsx__WEBPACK_IMPORTED_MODULE_6__[\"default\"]", None);
        source.replace_static(839, 843, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(844, 856, "_d0_f5_jsx__WEBPACK_IMPORTED_MODULE_7__[\"default\"]", None);
        source.replace_static(889, 893, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(894, 906, "_d0_f6_jsx__WEBPACK_IMPORTED_MODULE_8__[\"default\"]", None);
        source.replace_static(939, 943, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(944, 956, "_d0_f7_jsx__WEBPACK_IMPORTED_MODULE_9__[\"default\"]", None);
        source.replace_static(989, 993, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(994, 1006, "_d0_f8_jsx__WEBPACK_IMPORTED_MODULE_10__[\"default\"]", None);
        source.replace_static(1032, 1047, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1032, 1047, "", None);
        source.replace_static_with_enforce(1053, 1054, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/d0/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!***********************!*\\\n  !*** ./src/d0/f0.jsx ***!\n  \\***********************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d0/f0.jsx */ \"./src/d0/d0/f0.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d0/f0.jsx';\n// import Component__1 from './d0/f1.jsx'\n// import Component__2 from './d0/f2.jsx'\n// import Component__3 from './d0/f3.jsx'\n// import Component__4 from './d0/f4.jsx'\n// import Component__5 from './d0/f5.jsx'\n// import Component__6 from './d0/f6.jsx'\n// import Component__7 from './d0/f7.jsx'\n// import Component__8 from './d0/f8.jsx'\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            /*#__PURE__*/ _jsx(Component__0, {}),\n            /*#__PURE__*/ _jsx(Component__1, {}),\n            /*#__PURE__*/ _jsx(Component__2, {}),\n            /*#__PURE__*/ _jsx(Component__3, {}),\n            /*#__PURE__*/ _jsx(Component__4, {}),\n            /*#__PURE__*/ _jsx(Component__5, {}),\n            /*#__PURE__*/ _jsx(Component__6, {}),\n            /*#__PURE__*/ _jsx(Component__7, {}),\n            /*#__PURE__*/ _jsx(Component__8, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/d0/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/d0/f0.jsx\"],\"sourcesContent\":[\"\\n  import React from 'react'\\n  import Icon  from '@icon-park/react/es/all';\\n\\n  import Component__0 from './d0/f0.jsx'\\n// import Component__1 from './d0/f1.jsx'\\n// import Component__2 from './d0/f2.jsx'\\n// import Component__3 from './d0/f3.jsx'\\n// import Component__4 from './d0/f4.jsx'\\n// import Component__5 from './d0/f5.jsx'\\n// import Component__6 from './d0/f6.jsx'\\n// import Component__7 from './d0/f7.jsx'\\n// import Component__8 from './d0/f8.jsx'\\n  function Navbar({ show }) {\\n    return (\\n      <div>\\n      <Component__0/>\\n<Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/>\\n      </div>\\n    )\\n  }\\n  \\n  export default Navbar\\n  \"],\"names\":[\"React\",\"Component__0\",\"Navbar\",\"param\",\"show\",\"Component__1\",\"Component__2\",\"Component__3\",\"Component__4\",\"Component__5\",\"Component__6\",\"Component__7\",\"Component__8\"],\"mappings\":\";AACE,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACxC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACvC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAChB,qBACE,MAAC;;0BACD,KAACH;0BACP,KAACI;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;0BACD,KAACC;;;AAGC;AAEA,eAAeV,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(544, 549, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(605, 609, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(610, 622, "_d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(655, 659, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(705, 709, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(755, 759, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(805, 809, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(855, 859, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(905, 909, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(955, 959, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1005, 1009, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(1048, 1063, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(1048, 1063, "", None);
        source.replace_static_with_enforce(1069, 1070, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\"./src/f0.jsx\"").boxed(),
      RawStringSource::from_static(": ").boxed(),
      RawStringSource::from_static("\n/*!********************!*\\\n  !*** ./src/f0.jsx ***!\n  \\********************/\n").boxed(),
      RawStringSource::from_static("(function (__unused_webpack_module, __webpack_exports__, __webpack_require__) {\n").boxed(),
      RawStringSource::from_static("__webpack_require__.r(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("__webpack_require__.d(__webpack_exports__, {\n  \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n});\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! react/jsx-runtime */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/jsx-runtime.js\");\n").boxed(),
      RawStringSource::from_static("/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! react */ \"../../node_modules/.pnpm/react@18.2.0/node_modules/react/index.js\");\n/* ESM import */var react__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(react__WEBPACK_IMPORTED_MODULE_1__);\n").boxed(),
      RawStringSource::from_static("/* ESM import */var _d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./d0/f0.jsx */ \"./src/d0/f0.jsx\");\n").boxed(),
      {
        let mut source = ReplaceSource::new(
          SourceMapSource::new(SourceMapSourceOptions {
            value: "import { jsx as _jsx, jsxs as _jsxs } from \"react/jsx-runtime\";\nimport React from 'react';\nimport Component__0 from './d0/f0.jsx';\n// import Component__1 from './d0/f1.jsx'\n// import Component__2 from './d0/f2.jsx'\n// import Component__3 from './d0/f3.jsx'\n// import Component__4 from './d0/f4.jsx'\n// import Component__5 from './d0/f5.jsx'\n// import Component__6 from './d0/f6.jsx'\n// import Component__7 from './d0/f7.jsx'\n// import Component__8 from './d0/f8.jsx'\nfunction Navbar(param) {\n    var show = param.show;\n    return /*#__PURE__*/ _jsxs(\"div\", {\n        children: [\n            \"9\",\n            /*#__PURE__*/ _jsx(Component__0, {})\n        ]\n    });\n}\nexport default Navbar;\n",
            name: "builtin:swc-loader??ruleSet[1].rules[1].use[0]!/rspack-ecosystem-benchmark/cases/10000/src/f0.jsx",
            source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"/rspack-ecosystem-benchmark/cases/10000/src/f0.jsx\"],\"sourcesContent\":[\"\\nimport React from 'react'\\nimport Icon  from '@icon-park/react/es/all';\\n\\nimport Component__0 from './d0/f0.jsx'\\n// import Component__1 from './d0/f1.jsx'\\n// import Component__2 from './d0/f2.jsx'\\n// import Component__3 from './d0/f3.jsx'\\n// import Component__4 from './d0/f4.jsx'\\n// import Component__5 from './d0/f5.jsx'\\n// import Component__6 from './d0/f6.jsx'\\n// import Component__7 from './d0/f7.jsx'\\n// import Component__8 from './d0/f8.jsx'\\nfunction Navbar({ show }) {\\nreturn (\\n  <div>\\n  9\\n  <Component__0/>\\n{/* <Component__1/>\\n<Component__2/>\\n<Component__3/>\\n<Component__4/>\\n<Component__5/>\\n<Component__6/>\\n<Component__7/>\\n<Component__8/> */}\\n  </div>\\n)\\n}\\n\\nexport default Navbar\\n\\n\\n\"],\"names\":[\"React\",\"Component__0\",\"Navbar\",\"param\",\"show\"],\"mappings\":\";AACA,OAAOA,WAAW,QAAO;AAGzB,OAAOC,kBAAkB,cAAa;AACtC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,yCAAyC;AACzC,SAASC,OAAOC,KAAQ;QAARA,AAAEC,OAAFD,MAAEC;IAClB,qBACE,MAAC;;YAAI;0BAEL,KAACH;;;AAWH;AAEA,eAAeC,OAAM\"}").unwrap(),
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          }).boxed()
        );
        source.replace_static(0, 63, "", None);
        source.replace_static(64, 90, "", None);
        source.replace_static(91, 130, "", None);
        source.replace_static(544, 549, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsxs)", None);
        source.replace_static(622, 626, "(0,react_jsx_runtime__WEBPACK_IMPORTED_MODULE_0__.jsx)", None);
        source.replace_static(627, 639, "_d0_f0_jsx__WEBPACK_IMPORTED_MODULE_2__[\"default\"]", None);
        source.replace_static(665, 680, "/* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = (", None);
        source.replace_static(665, 680, "", None);
        source.replace_static_with_enforce(686, 687, ");", None, ReplacementEnforce::Post);
        source.boxed()
      },
      RawStringSource::from_static("\n\n})").boxed(),
      RawStringSource::from_static(",\n").boxed(),
      RawStringSource::from_static("\n}").boxed(),
      RawStringSource::from_static(");\n").boxed(),
      RawStringSource::from_static("/************************************************************************/\n").boxed(),
      RawStringSource::from_static("// The module cache\nvar __webpack_module_cache__ = {};\n\n// The require function\nfunction __webpack_require__(moduleId) {\n\n// Check if module is in cache\nvar cachedModule = __webpack_module_cache__[moduleId];\nif (cachedModule !== undefined) {\nreturn cachedModule.exports;\n}\n// Create a new module (and put it into the cache)\nvar module = (__webpack_module_cache__[moduleId] = {\nid: moduleId,\nloaded: false,\nexports: {}\n});\n// Execute the module function\n__webpack_modules__[moduleId](module, module.exports, __webpack_require__);\n\n// Flag the module as loaded\nmodule.loaded = true;\n// Return the exports of the module\nreturn module.exports;\n\n}\n\n// expose the modules object (__webpack_modules__)\n__webpack_require__.m = __webpack_modules__;\n\n").boxed(),
      RawStringSource::from_static("/************************************************************************/\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/compat_get_default_export\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "// getDefaultExport function for compatibility with non-ESM modules\n__webpack_require__.n = (module) => {\n\tvar getter = module && module.__esModule ?\n\t\t() => (module['default']) :\n\t\t() => (module);\n\t__webpack_require__.d(getter, { a: getter });\n\treturn getter;\n};\n",
        "webpack/runtime/compat_get_default_export",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/define_property_getters\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "__webpack_require__.d = (exports, definition) => {\n\tfor(var key in definition) {\n        if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {\n            Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });\n        }\n    }\n};",
        "webpack/runtime/define_property_getters",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/has_own_property\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))",
        "webpack/runtime/has_own_property",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/make_namespace_object\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "// define __esModule on exports\n__webpack_require__.r = (exports) => {\n\tif(typeof Symbol !== 'undefined' && Symbol.toStringTag) {\n\t\tObject.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });\n\t}\n\tObject.defineProperty(exports, '__esModule', { value: true });\n};",
        "webpack/runtime/make_namespace_object",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/node_module_decorator\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "__webpack_require__.nmd = (module) => {\n  module.paths = [];\n  if (!module.children) module.children = [];\n  return module;\n};",
        "webpack/runtime/node_module_decorator",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/on_chunk_loaded\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "var deferred = [];\n__webpack_require__.O = (result, chunkIds, fn, priority) => {\n\tif (chunkIds) {\n\t\tpriority = priority || 0;\n\t\tfor (var i = deferred.length; i > 0 && deferred[i - 1][2] > priority; i--)\n\t\t\tdeferred[i] = deferred[i - 1];\n\t\tdeferred[i] = [chunkIds, fn, priority];\n\t\treturn;\n\t}\n\tvar notFulfilled = Infinity;\n\tfor (var i = 0; i < deferred.length; i++) {\n\t\tvar [chunkIds, fn, priority] = deferred[i];\n\t\tvar fulfilled = true;\n\t\tfor (var j = 0; j < chunkIds.length; j++) {\n\t\t\tif (\n\t\t\t\t(priority & (1 === 0) || notFulfilled >= priority) &&\n\t\t\t\tObject.keys(__webpack_require__.O).every((key) => (__webpack_require__.O[key](chunkIds[j])))\n\t\t\t) {\n\t\t\t\tchunkIds.splice(j--, 1);\n\t\t\t} else {\n\t\t\t\tfulfilled = false;\n\t\t\t\tif (priority < notFulfilled) notFulfilled = priority;\n\t\t\t}\n\t\t}\n\t\tif (fulfilled) {\n\t\t\tdeferred.splice(i--, 1);\n\t\t\tvar r = fn();\n\t\t\tif (r !== undefined) result = r;\n\t\t}\n\t}\n\treturn result;\n};\n",
        "webpack/runtime/on_chunk_loaded",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/rspack_version\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "__webpack_require__.rv = () => (\"1.6.0-beta.1\")",
        "webpack/runtime/rspack_version",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/jsonp_chunk_loading\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "\n      // object to store loaded and loading chunks\n      // undefined = chunk not loaded, null = chunk preloaded/prefetched\n      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded\n      var installedChunks = {\"main\": 0,};\n      __webpack_require__.O.j = (chunkId) => (installedChunks[chunkId] === 0);\n// install a JSONP callback for chunk loading\nvar webpackJsonpCallback = (parentChunkLoadingFunction, data) => {\n\tvar [chunkIds, moreModules, runtime] = data;\n\t// add \"moreModules\" to the modules object,\n\t// then flag all \"chunkIds\" as loaded and fire callback\n\tvar moduleId, chunkId, i = 0;\n\tif (chunkIds.some((id) => (installedChunks[id] !== 0))) {\n\t\tfor (moduleId in moreModules) {\n\t\t\tif (__webpack_require__.o(moreModules, moduleId)) {\n\t\t\t\t__webpack_require__.m[moduleId] = moreModules[moduleId];\n\t\t\t}\n\t\t}\n\t\tif (runtime) var result = runtime(__webpack_require__);\n\t}\n\tif (parentChunkLoadingFunction) parentChunkLoadingFunction(data);\n\tfor (; i < chunkIds.length; i++) {\n\t\tchunkId = chunkIds[i];\n\t\tif (\n\t\t\t__webpack_require__.o(installedChunks, chunkId) &&\n\t\t\tinstalledChunks[chunkId]\n\t\t) {\n\t\t\tinstalledChunks[chunkId][0]();\n\t\t}\n\t\tinstalledChunks[chunkId] = 0;\n\t}\n\treturn __webpack_require__.O(result);\n};\n\nvar chunkLoadingGlobal = self[\"webpackChunk_10000\"] = self[\"webpackChunk_10000\"] || [];\nchunkLoadingGlobal.forEach(webpackJsonpCallback.bind(null, 0));\nchunkLoadingGlobal.push = webpackJsonpCallback.bind(null, chunkLoadingGlobal.push.bind(chunkLoadingGlobal));\n",
        "webpack/runtime/jsonp_chunk_loading",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("// webpack/runtime/rspack_unique_id\n").boxed(),
      RawStringSource::from_static("(() => {\n").boxed(),
      OriginalSource::new(
        "__webpack_require__.ruid = \"bundler=rspack@1.6.0-beta.1\";\n",
        "webpack/runtime/rspack_unique_id",
      ).boxed(),
      RawStringSource::from_static("\n})();\n").boxed(),
      RawStringSource::from_static("/************************************************************************/\n").boxed(),
      RawStringSource::from_static("// startup\n// Load entry module and return exports\n// This entry module depends on other loaded chunks and execution need to be delayed\nvar __webpack_exports__ = __webpack_require__.O(undefined, [\"vendors-node_modules_pnpm_react-dom_18_2_0_react_18_2_0_node_modules_react-dom_client_js-node-ca348c\"], function() { return __webpack_require__(\"./index.jsx\") });\n__webpack_exports__ = __webpack_require__.O(__webpack_exports__);\n").boxed(),
      RawStringSource::from_static("})()\n").boxed(),
      RawStringSource::from_static(";").boxed(),
    ]).boxed()
});

pub fn benchmark_repetitive_react_components_map(b: &mut Bencher) {
  let source = REPETITIVE_1K_REACT_COMPONENTS_SOURCE.clone();

  b.iter(|| {
    std::hint::black_box(source.map(&ObjectPool::default(), &MapOptions::default()));
  });
}

pub fn benchmark_repetitive_react_components_source(b: &mut Bencher) {
  let source = REPETITIVE_1K_REACT_COMPONENTS_SOURCE.clone();

  b.iter(|| {
    std::hint::black_box(source.source());
  });
}
