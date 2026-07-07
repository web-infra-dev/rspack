import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const mixedDisabledRequired = require("./dep.js");
export const mixedDisabledResolved = require.resolve("path");
export const mixedDisabledUnknownMember = require.a;
