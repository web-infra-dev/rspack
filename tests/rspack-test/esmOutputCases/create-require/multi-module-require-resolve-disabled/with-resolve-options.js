import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const resolveWithOptions = require.resolve("path", { paths: [] });
