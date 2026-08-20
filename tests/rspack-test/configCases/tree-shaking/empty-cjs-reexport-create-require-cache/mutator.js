import { createRequire } from "node:module";

const createdRequire = createRequire(import.meta.url);
createdRequire.cache["./empty.js"].exports.cacheValue = "create-require-cache";
