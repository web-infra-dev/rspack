exports.abc = {};

for (let i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	exports.abc[i.name] = i.name;
}
