import { JsDependency } from "@rspack/binding";

// 目前不处理 EntryDependency 的继承层级
// 现在EntryDependency 目前唯一的用处就是作为 addInclude 函数的参数
// EntryDependency 的存在原因是 Rust 侧 EntryDependency 和 webpack 的 EntryDependency 构造不一致
// 故 js 侧通过 EntryDependency 占位，直到构造信息完整时才进行构建
export class EntryDependency {
	#inner?: JsDependency;

	request: string;

	static __to_binding(dependency: EntryDependency): JsDependency {
		if (!dependency.#inner) {
			throw new Error("TODO");
		}
		return dependency.#inner;
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
