import { Compilation } from "./compilation";

export class RspackStats {
	inner_value: any;
	constructor(value: any) {}
	toJson() {
		return this.inner_value || {};
	}
}
