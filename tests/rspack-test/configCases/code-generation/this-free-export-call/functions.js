export function plainFunction() {
	return "plain";
}

export const arrowFunction = () => this;

export const functionExpression = function () {
	return "expression";
};

export function nestedFunctionThis() {
	return function () {
		return this;
	};
}

export function nestedClassThis() {
	return class {
		value = this;
		static {
			this.initialized = true;
		}
		method() {
			return this;
		}
	};
}

export { laterFunction as renamedLater };

function laterFunction() {
	return "later";
}

export function usesThis() {
	return this;
}

export function arrowThis() {
	return (() => this)();
}

export function parameterThis(value = this) {
	return value;
}

export function computedKeyThis() {
	return class {
		[String(this)]() {}
	};
}

export function reassigned() {
	return "original";
}

export function reassign() {
	reassigned = function () {
		return this;
	};
}
