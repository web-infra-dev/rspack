module.exports.abc = {};

for (let i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	module.exports.abc[i.name] = i.name;
}
