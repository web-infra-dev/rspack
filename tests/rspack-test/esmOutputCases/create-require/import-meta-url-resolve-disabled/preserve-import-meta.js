import { createRequire } from "node:module";

const req = createRequire(import.meta.url);
var varReq = createRequire(import.meta.url);
let letReq = createRequire(import.meta.url);

export const preservedResolved = req.resolve("path");
export const preservedVarResolved = varReq.resolve("path");
export const preservedLetResolved = letReq.resolve("path");
