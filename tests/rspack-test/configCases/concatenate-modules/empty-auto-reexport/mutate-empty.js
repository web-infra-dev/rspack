"use strict";

require("./mutated-empty").value = 42;
globalThis.emptyAutoReexportMutatorExecuted = true;
