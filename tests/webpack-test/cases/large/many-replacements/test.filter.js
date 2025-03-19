// module.exports = function (config) {
// 	return !process.env.CI;
// };

// Already pass this test, but this test is too slow, and create a lot of big files, so we always skip this test
module.exports = () => "Already pass this test, but this test is too slow, and create a lot of big files, so we always skip this test"
