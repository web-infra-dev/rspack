module.exports = [
	{
		rebuildChangeFile: "./src/f0.jsx",
		generateContent: function (originalContent, runTimes) {
			return (
				`import "data:text/javascript,export default ${runTimes}";
` + originalContent
			);
		}
	}
];
