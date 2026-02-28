import * as a from "./star*/a.js";

export const staticA = a;
export const dynamicA = await import("./star*/a.js")
