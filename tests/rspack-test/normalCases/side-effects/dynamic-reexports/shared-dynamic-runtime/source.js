const [name, value] = __resourceQuery.slice(1).split("=");

Object(exports)[name] = Number(value);
Object(exports).local = "from dynamic source";

if (name === "a") {
	Object(exports).setA = next => {
		Object(exports).a = next;
	};
	Object(exports).default = "excluded default";
}

if (name === "b") {
	Object(exports).a = 999;
}
