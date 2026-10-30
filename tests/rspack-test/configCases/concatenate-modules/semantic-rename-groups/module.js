var value = "outer";
var value;
let caught;
try {
	throw "caught";
} catch (value) {
	var value = "changed";
	({ value = "default" } = {});
	caught = () => ({ value });
}

function shadowed(value = "parameter", read = () => value) {
	var value;
	value = "body";
	return [value, read()];
}

let independent;
try {
	throw "independent";
} catch (value) {
	independent = () => value;
}

export { value, caught, shadowed, independent };
