const normalizeErrors = require("./normalizeErrors");

module.exports = stats => normalizeErrors([...stats.compilation.errors]);
