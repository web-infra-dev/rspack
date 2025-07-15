module.exports.abc = {};

for (const i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	module.exports.abc[i.name] = i.name;
}
