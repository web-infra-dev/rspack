import { getValue } from "./value";

class View {}

export const Used = class Used extends View {
	static getValue = getValue;
};

export const UnusedWithConstructor = class UnusedWithConstructor extends View {
	constructor() {
		super();
	}

	static getValue = getValue;
};

export const UnusedWithStaticBlock = class UnusedWithStaticBlock extends View {
	static {}

	static getValue = getValue;
};
