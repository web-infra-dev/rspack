if (Math.random() < 2) {
  require("./_require-conditional-execution-register1.js");
} else {
  require("./_require-conditional-execution-register2.js");
}
module.exports = require("./_require-conditional-execution.js");
