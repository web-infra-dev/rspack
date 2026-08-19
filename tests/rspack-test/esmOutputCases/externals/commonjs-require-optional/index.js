let value;
let missing;

try {
	value = require("optional-external").value;
} catch {
	value = -1;
}

try {
	require("missing-external");
	missing = false;
} catch {
	missing = true;
}

if (value !== 42 || !missing) {
	throw new Error("optional external should keep working through the wrapper path");
}

export { value, missing };
