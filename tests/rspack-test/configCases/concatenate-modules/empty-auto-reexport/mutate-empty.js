"use strict";

const target = require("./mutated-empty");
target.value = 42;
globalThis.emptyAutoReexportMutatedValue = target.value;
