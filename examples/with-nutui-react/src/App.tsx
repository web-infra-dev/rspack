import { useState } from "react";
import reactLogo from "./assets/react.svg";
import "./App.scss";
import { Button } from "@nutui/nutui-react";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">
      <div>
        <a className="logo-a" href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1 className="title">Rspack + React + TypeScript</h1>
      <div className="card">
        <button className="count-btn" onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">Click on the Rspack and React logos to learn more</p>
      <Button type="primary">主要按钮</Button>
    </div>
  );
}

export default App;
