import { importedPrivate } from "./imported-private";
import { sharedValue, trace } from "./shared";

trace.push("imported");

export const foo = `${sharedValue}:${importedPrivate}`;
export const bar = "DROP_IMPORTED_BAR_SENTINEL";
export default 42;
