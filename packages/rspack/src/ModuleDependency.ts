import { Dependency } from "./Dependency";

export class ModuleDependency extends Dependency {
	#request: string;

	constructor(request: string) {
		super();
		this.#request = request;
	}

	get request(): string {
		return this.#request;
	}
}
