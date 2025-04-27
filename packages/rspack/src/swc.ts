import type { TransformOutput } from "@rspack/binding";
import binding from "@rspack/binding";
import type { JsMinifyOptions, Options as TransformOptions } from "@swc/types";

export type { TransformOutput, TransformOptions, JsMinifyOptions };

export async function minify(
	source: string,
	options?: JsMinifyOptions
): Promise<TransformOutput> {
	const _options = JSON.stringify(options || {});

	return binding.minify(source, _options);
}

export async function transform(
	source: string,
	options?: TransformOptions
): Promise<TransformOutput> {
	const _options = JSON.stringify(options || {});

	return binding.transform(source, _options);
}
