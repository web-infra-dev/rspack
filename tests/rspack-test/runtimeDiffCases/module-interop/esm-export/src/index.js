import * as star from "./esm/export-star";
import { a, b, c } from "./esm/export";
import { d, e } from "./esm/export-as";
import { AAA } from "./esm/export-class";
import { aaa } from "./esm/export-function";

import _default from "./esm/export-default";
import _default_fn from "./esm/export-default-function";
import _default_fn_params from "./esm/export-default-function-params";
import _default_fn_async from "./esm/export-default-function-async";
import _default_fn_generator from "./esm/export-default-function-generator";
import _default_fn_nameless from "./esm/export-default-function-nameless";
import _default_class_nameless from "./esm/export-default-class-nameless";
import _default_class from "./esm/export-default-class";
import _default_expression from "./esm/export-default-expression";
import "./esm/variables";

// prevent global use strict
require("./no-strict");

console.log(star);
console.log(star.a);

console.log(AAA);
console.log(aaa);

console.log(a);
console.log(b);
console.log(c);
console.log(d);
console.log(e);
console.log(_default);
console.log(_default_class);
console.log(_default_fn);
console.log(_default_fn_params);
console.log(_default_fn_async);
console.log(_default_fn_generator);
console.log(_default_expression);
