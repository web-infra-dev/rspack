export function café(value) {
  const prefix = "😀雪"; const suffix = value.length;
  return prefix + value + suffix;
}

export function résumé(value) {
  return café(value).toUpperCase();
}
