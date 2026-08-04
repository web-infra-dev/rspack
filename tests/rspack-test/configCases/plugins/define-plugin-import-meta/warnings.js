module.exports = [
	[/Accessing unknown `import.meta` property 'env' is replaced with undefined\./],
	[
		{
			message:
				/Accessing unknown `import.meta` property 'unknownProperty' is replaced with undefined\.[\s\S]*if \(FOO\)/
		}
	]
];
