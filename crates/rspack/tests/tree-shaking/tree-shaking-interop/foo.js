if (process.env.NODE_ENV !== "production") {
	const res = require("./a");
	module.exports = res;
} else {
	const c = require("./b");
	module.exports = c;
}

// export default function () {}
