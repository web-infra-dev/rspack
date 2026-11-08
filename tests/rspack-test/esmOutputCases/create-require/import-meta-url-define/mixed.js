import { createRequire } from "node:module";

const req = createRequire(import.meta.url);

export const bundledValue = req("./dependency.cjs");
export const unknownValue = req.unknown;
