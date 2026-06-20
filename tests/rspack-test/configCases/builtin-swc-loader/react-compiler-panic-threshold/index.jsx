import { useRef } from 'react';

const App = () => {
  const ref = useRef(1)
  return (
    <div className="content">
      <h1>Rsbuild with React</h1>
      <p>Start building amazing things with Rsbuild. ${ref.current}</p>
    </div>
  );
};

it("should emit react compiler output with swc-loader", () => {
  const fs = require("fs");
  const source = fs.readFileSync(__filename, "utf-8");
  expect(source).not.toContain(["react", "compiler-runtime"].join("/"));
  expect(source).not.toContain(["react", "memo_cache_sentinel"].join("."));
});
