import { createRequire } from "node:module";

export const inlineResolved = createRequire(import.meta.url).resolve("path");
