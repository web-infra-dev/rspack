export default function (source, map) {
  this.cacheable(false);
  const data = { resource: this.resource };
  this.trackReference(data);
  // The parser drops this additionalData on a Rust thread after the loader
  // returns. It must be deleted on this worker, even if another worker exits.
  this.callback(null, source, map, data);
}
