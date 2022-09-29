import { RspackCompilation } from "./rspack";

export class RspackStats {
	inner_value: any;
	constructor(value: any) {}
	toJson() {
		return this.inner_value || {};
	}
}
