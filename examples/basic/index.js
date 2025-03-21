import "./lib";
import camelCase from "https://esm.sh/lodash-es@4.17.21/camelCase";
import React from "https://esm.sh/react@18.2.0";

// Also demonstrate CommonJS require from HTTP URLs
// NOTE: Requires need to be top-level to be correctly processed as static imports
const upperFirst = require("https://esm.sh/lodash-es@4.17.21/upperFirst");

console.log("React version:", React.version);
console.log("Testing lodash camelCase:", camelCase("hello world"));
console.log("Testing lodash upperFirst:", upperFirst("hello world"));
