import { useRef } from 'react';

const App = () => {
  const ref = useRef(1);
  (ref.current as number) = 2;
  return (
    <div className="content">
      <h1>Rsbuild with React</h1>
      <p>Start building amazing things with Rsbuild. ${ref.current}</p>
    </div>
  );
};

it("should build successfully with react compiler enabled (paren assignment target)", () => {
  const fs = require("fs");
  const source = fs.readFileSync(__filename, "utf-8");
  expect(source).toContain(["Rsbuild", "with", "React"].join(" "));
});
