// Accessing the module object keeps this case behind a wrapper boundary.
this.value = true;
module.forceWrapper = true;
module.exports = this.value;
