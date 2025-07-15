exports.abc = {};

for (const i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	exports.abc[i.name] = i.name;
}
