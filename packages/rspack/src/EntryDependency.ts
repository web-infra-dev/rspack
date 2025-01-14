import { ModuleDependency } from "./ModuleDependency";

export class EntryDependency extends ModuleDependency {
	constructor(request: string) {
		super(request);
	}
}
