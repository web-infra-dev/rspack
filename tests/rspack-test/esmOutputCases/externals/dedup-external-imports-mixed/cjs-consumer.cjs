// CJS module - will NOT be scope-hoisted, causing its fs dep
// to go through init fragment path instead of raw_import_stmts
const fs = require('fs')
exports.cjsResult = fs.readFileSync
