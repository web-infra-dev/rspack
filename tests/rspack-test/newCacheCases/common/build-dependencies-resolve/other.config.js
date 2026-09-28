require("./dep");
// lib/a will be unusable because lib does not export it.
require("lib/a");
require("lib/b");
// lib/src/index will be unusable because lib does not export it.
require("lib/src/index");
