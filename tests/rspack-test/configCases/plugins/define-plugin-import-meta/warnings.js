module.exports = [
	[/Accessing unknown property 'env' is replaced with undefined\./],
	[
		{
			message:
				/Accessing unknown property 'unknownProperty' is replaced with undefined\.[\s\S]*if \(FOO\)/
		}
	]
];
