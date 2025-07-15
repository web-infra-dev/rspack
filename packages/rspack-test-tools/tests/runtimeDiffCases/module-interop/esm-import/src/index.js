import defaultExport from "./esm/default";
import defaultExport4, * as defaultExport5 from "./esm/default-multiple-rename";
import { default as defaultExport2 } from "./esm/default-rename";
import defaultExport3, { h as _h, g } from "./esm/export-default-multiple";
import { c, d } from "./esm/export-multiple";
import { f as _f, e } from "./esm/export-multiple-rename";
import { b as _b } from "./esm/export-rename";
import { a } from "./esm/export-single";
import * as starExport from "./esm/star";
import "./esm/unuse";

// prevent global use strict
require("./no-strict");

console.log(starExport);

console.log(defaultExport);
console.log(defaultExport2);
console.log(defaultExport3);
console.log(defaultExport4);
console.log(defaultExport5);

console.log(a);
console.log(_b);
console.log(c);
console.log(d);
console.log(e);
console.log(_f);
console.log(g);
console.log(_h);
