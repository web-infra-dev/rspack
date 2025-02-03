// In webpack this file is errors.js, these should be two errors,
// but seems warnings are more appropriate here, since the diagnostic name is UnsupportedFeatureWarning.
module.exports = [
	[/Cannot statically analyse/, /\[4:\d+\]/],
	[/Cannot statically analyse/, /\[12:\d+\]/]
];
