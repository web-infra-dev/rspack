const [name, value] = __resourceQuery.slice(1).split("=");

Object(exports)[name] = Number(value);

if (name === "a") {
	Object(exports).setA = function (next) {
		Object(exports).a = next;
	};
	Object(exports).default = "excluded default";
}

if (name === "b") {
	Object(exports).a = 999;
}
