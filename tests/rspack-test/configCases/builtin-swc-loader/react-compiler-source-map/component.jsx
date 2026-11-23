function format(value) {
  return `${value}`;
}

export function Counter({ count }) {
  const label = format("react-compiler-body-marker");
  return (
    <div title={label}>{count}</div>
  );
}
