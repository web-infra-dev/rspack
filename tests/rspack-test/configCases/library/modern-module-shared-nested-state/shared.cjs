const counter = require('./counter.cjs');
module.exports = { counter };
module.exports.load = () => import('./async.cjs');
