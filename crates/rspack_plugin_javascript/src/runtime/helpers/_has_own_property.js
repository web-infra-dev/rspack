function __has_own_property__(o, p) {
  return Object.prototype.hasOwnProperty.call(o, p)
}

globalThis.rs.has_own_property = globalThis.rs.has_own_property || __has_own_property__
