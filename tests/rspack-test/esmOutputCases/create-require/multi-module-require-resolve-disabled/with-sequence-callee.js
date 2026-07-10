import { createRequire } from "node:module";

const require = (0, createRequire)(import.meta.url);

export const sequenceResolved = require.resolve("path");
