export * from "../index";
export { BrowserHttpImportEsmPlugin } from "./BrowserHttpImportEsmPlugin";

import { fs, volume } from "./fs";
export const builtinMemFs = {
	fs,
	volume
};
