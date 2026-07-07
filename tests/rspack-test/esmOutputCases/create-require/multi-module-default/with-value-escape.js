import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const escapedRequire = require;

export const escapedRequireType = typeof escapedRequire;
