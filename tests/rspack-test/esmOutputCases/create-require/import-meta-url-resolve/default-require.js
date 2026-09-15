import moduleDefault from "node:module";

const defaultRequire = moduleDefault.createRequire(import.meta.url);

export const defaultUnknown = defaultRequire.unknown;
