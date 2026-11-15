import * as ns from "./lib?namespace";
import lib from "./lib?default";

export const readNamespace = name => ns[name];
export const readDefault = name => lib[name];
export const namespaceUsedExports = ns.usedExports;
export const defaultUsedExports = lib.usedExports;
