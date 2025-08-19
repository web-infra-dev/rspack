/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const optimize: (
	a: number,
	b: number,
	c: number,
	d: number
) => [number, number];
export const parse_webpack_chunk: (a: number, b: number) => [number, number];
export const get_webpack_module_info: (
	a: number,
	b: number,
	c: number,
	d: number
) => [number, number];
export const get_webpack_dependency_graph: (
	a: number,
	b: number
) => [number, number];
export const get_webpack_dependency_tree: (
	a: number,
	b: number,
	c: number,
	d: number
) => [number, number];
export const optimize_with_prune_result_json: (
	a: number,
	b: number,
	c: number,
	d: number
) => [number, number];
export const __wbindgen_export_0: WebAssembly.Table;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (
	a: number,
	b: number,
	c: number,
	d: number
) => number;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
