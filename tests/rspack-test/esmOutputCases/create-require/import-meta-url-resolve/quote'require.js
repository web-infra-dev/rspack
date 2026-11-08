import { createRequire } from "node:module";

const quotedRequire = createRequire(import.meta.url);

export const quotedUnknown = quotedRequire.unknown;
