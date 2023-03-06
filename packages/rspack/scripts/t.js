const check = require("../src/config/schema.check");
const path = require("path");
console.log(check);
const d = check({
	// bad: 1
});
console.log(d, check);
