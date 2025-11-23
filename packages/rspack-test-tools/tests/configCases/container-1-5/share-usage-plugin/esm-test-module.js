import { clone, cloneDeep, merge, mergeWith } from "lodash-es";
import defaultExport, { useState, useEffect } from "react";

export function useClone(obj) {
	return clone(obj);
}

export function useMerge(obj1, obj2) {
	return merge(obj1, obj2);
}

export { default as ReactDefault } from "react";

export * from "./barrel-exports.js";

export default function mainFunction() {
	return "ESM default export";
}
