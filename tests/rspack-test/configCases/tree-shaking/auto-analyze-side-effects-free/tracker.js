export const events = [];

export function record(label) {
  events.push(label);
  return events.length;
}
