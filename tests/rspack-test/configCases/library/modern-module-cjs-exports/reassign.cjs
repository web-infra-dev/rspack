const original = module.exports;
exports = { value: 2 };

function update() {
  exports = { value: 3 };
}
update();

module.exports = { original, reassigned: exports };
