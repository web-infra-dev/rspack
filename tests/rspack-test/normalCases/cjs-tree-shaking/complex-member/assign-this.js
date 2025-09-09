this.abc = {};

for (let i of [
	{
		name: "a"
	},
	{
		name: "b"
	}
]) {
	this.abc[i.name] = i.name;
}
