import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const mixedRequired = require("./dep.js");
export const mixedResolved = require.resolve("./dep.js");
export const mixedUnknownMember = require.a;
