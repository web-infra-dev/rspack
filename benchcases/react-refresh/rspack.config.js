const path = require("path");
const HmrBenchmarkPlugin = require("./plugins/hmr-benchmark-plugin");
/**
 * @type {import('@rspack/cli').Configuration}
 */
let index = 0;
module.exports = {
  mode: "development",
  entry: { main: "./src/index.tsx" },
  builtins: {
    html: [{template: './index.html'}],
    define: {
      "process.env.NODE_ENV": "'development'",
    },
  },
  plugins: [
    new HmrBenchmarkPlugin({
      [path.resolve(__dirname, "./src/App.tsx")]: () => {
        return `
        import { lazy, Suspense } from 'react'
import { ArrowFunction } from './ArrowFunction'
import ClassDefault from './ClassDefault'
import { ClassNamed } from './ClassNamed'
import FunctionDefault from './FunctionDefault'
import { FunctionNamed } from './FunctionNamed'

const LazyComponent = lazy(() => import('./LazyComponent'))

function App() {
  return (
    <div>
      <div>${index++}</div>
      <ClassDefault />
      <ClassNamed />
      <FunctionDefault />
      <FunctionNamed />
      <ArrowFunction />
      <Suspense fallback={<h1>Loading</h1>}>
        <LazyComponent />
      </Suspense>
    </div>
  )
}

export default App

`
      }
    })
  ]
};
