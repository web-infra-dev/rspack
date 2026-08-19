const external = require("./safe.cjs");

process.__mixedExternalValues.push(external);

export const value = external.value;
