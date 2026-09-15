require.context(request, false, /\.js$/, "unknown-mode");

const ctx = require.context("./dir", false, /\.js$/, "unknown-mode");
module.exports = ctx;
