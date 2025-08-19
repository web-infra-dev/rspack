/* tslint:disable */
/* eslint-disable */
export function optimize(source: string, config: string): string;
export function parse_webpack_chunk(content: string): string;
export function get_webpack_module_info(
	content: string,
	module_key: string
): string;
export function get_webpack_dependency_graph(content: string): string;
export function get_webpack_dependency_tree(
	content: string,
	start_module_id: string
): string;
export function optimize_with_prune_result_json(
	source: string,
	config: string
): string;
