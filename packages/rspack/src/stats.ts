import * as binding from "@rspack/binding";
import { Compilation } from "./compilation";

export class Stats {
	compilation: Compilation;
	// remove this when support delegate compilation to rust side
	#statsJson: binding.StatsCompilation;

	constructor(compilation: Compilation, statsJson: binding.StatsCompilation) {
		this.compilation = compilation;
		this.#statsJson = statsJson;
	}

	toJson() {
		return this.#statsJson;
	}
}
