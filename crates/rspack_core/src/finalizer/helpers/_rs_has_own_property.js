rs._has_own_property = (o, p) => {
  return Object.prototype.hasOwnProperty.call(o, p);
};
