import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const resolvePaths = require.resolve.paths("path");
