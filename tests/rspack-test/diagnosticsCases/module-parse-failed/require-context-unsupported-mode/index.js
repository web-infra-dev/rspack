const ctx = require.context("./dir", false, /\.js$/, "unknown-mode");
module.exports = ctx;
