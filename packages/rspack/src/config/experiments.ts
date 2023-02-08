import { RawExperiments } from "@rspack/binding";

export function resolveExperiments(
	experiments?: RawExperiments
): RawExperiments {
	return {
		lazyCompilation: experiments?.lazyCompilation ?? false,
		incrementalRebuild: experiments?.incrementalRebuild ?? false
	};
}
