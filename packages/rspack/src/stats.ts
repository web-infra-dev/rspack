import { Compilation } from "./compilation";

export class Stats {
	compilation: Compilation;
	// remove this when support delegate compilation to rust side
	stats: any;
	constructor(compilation: Compilation, stats: any) {
		this.compilation = compilation;
		this.stats = stats;
	}
	toJson() {
		return this.stats;
	}
}
