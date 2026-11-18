export default function (buffer) {
  Atomics.wait(buffer, 0, 0);
  return 'ok';
}
