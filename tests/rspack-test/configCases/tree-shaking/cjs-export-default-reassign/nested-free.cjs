'use strict';
// The free identifier read is nested inside an otherwise-pure object literal.
// This must remain observable even when the export namespace is unused.
exports.value = { missingNested };
