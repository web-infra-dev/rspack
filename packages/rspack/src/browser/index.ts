export * from "../index";
export { BrowserImportEsmPlugin } from "./BrowserImportEsmPlugin";

import { fs, volume } from "./fs";
export const builtinMemFs = {
	fs,
	volume
};
