import { JsDependency, RawDependency } from "@rspack/binding";

// Currently, the inheritance hierarchy of EntryDependency is not handled.
// The only usage of EntryDependency for now is as a parameter for the addInclude function.
// The reason why EntryDependency exists is that the construction of EntryDependency on the Rust side is inconsistent with that of webpack's EntryDependency.
// Therefore, on the JavaScript side, EntryDependency is used as a placeholder until the construction information is complete for the actual build.
export class EntryDependency {
	#inner?: JsDependency;

	request: string;

	static __to_binding(dependency: EntryDependency): JsDependency {
		if (!dependency.#inner) {
			throw new Error(
				"The binding has not been attached to the EntryDependency."
			);
		}
		return dependency.#inner;
	}

	static __to_raw(dependency: EntryDependency): RawDependency {
		return {
			request: dependency.request
		};
	}

	static __attach_binding(dependency: EntryDependency, binding: JsDependency) {
		dependency.#inner = binding;
	}

	constructor(request: string) {
		this.request = request;
	}

	get type() {
		return "entry";
	}

	get category() {
		return "esm";
	}
}
