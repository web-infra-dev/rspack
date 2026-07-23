import { createRequire } from "node:module";

const req = createRequire(import.meta.url);

export const preservedResolved = req.resolve("path");
