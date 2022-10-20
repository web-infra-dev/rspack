import { Compilation } from "./compilation";
import { StatsDescription } from "@rspack/binding";

export class Stats {
	compilation: Compilation;
	// remove this when support delegate compilation to rust side
	stats: StatsDescription;
	constructor(compilation: Compilation, stats: StatsDescription) {
		this.compilation = compilation;
		this.stats = stats;
	}
	toJson() {
		return this.stats;
	}
}
