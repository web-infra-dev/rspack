globalThis.__template_coercion_effect__ = 0;
globalThis.__reassigned_pure_function_effect__ = 0;
globalThis.__reassigned_auto_function_effect__ = 0;

const value = {
	[Symbol.toPrimitive]() {
		globalThis.__template_coercion_effect__++;
		return "value";
	}
};

export const unusedTemplate = `${value}`;

let letFn = /*#__NO_SIDE_EFFECTS__*/ () => {};
letFn = () => {
	globalThis.__reassigned_pure_function_effect__++;
};

var varFn = /*#__NO_SIDE_EFFECTS__*/ () => {};
varFn = () => {
	globalThis.__reassigned_pure_function_effect__++;
};

export const unusedLetCall = letFn();
export const unusedVarCall = varFn();

function autoFn() {}
autoFn = () => {
	globalThis.__reassigned_auto_function_effect__++;
};

export function exportedAutoFn() {}
exportedAutoFn = () => {
	globalThis.__reassigned_auto_function_effect__++;
};

export const unusedAutoCall = autoFn();
export const unusedExportedAutoCall = exportedAutoFn();
