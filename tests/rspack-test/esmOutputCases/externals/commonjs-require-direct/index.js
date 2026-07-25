const external = require("external");
const reexported = require("./reexport.cjs");

export const value = external.value + reexported.value;
