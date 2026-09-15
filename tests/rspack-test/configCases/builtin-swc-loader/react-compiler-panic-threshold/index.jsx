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

it("should not emit react compiler output when compilation is skipped", () => {
  const fs = require("fs");
  const source = fs.readFileSync(__filename, "utf-8");
  expect(source).not.toContain(["react", "compiler-runtime"].join("/"));
  expect(source).not.toContain(["react", "memo_cache_sentinel"].join("."));
});
