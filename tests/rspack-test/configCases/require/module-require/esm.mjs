import { createRequire } from "module";
import { createRequire as nodeCreateRequire } from "node:module";

const require = createRequire(import.meta.url);
const nodeRequire = nodeCreateRequire(new URL("./foo/c.js", import.meta.url));

export const required = require("./a");
export const directRequired = createRequire(import.meta.url)("./c");
export const resolved = require.resolve("./b");
export const nodeRequired = nodeRequire("./a");
