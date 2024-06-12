export function App() {
  return (<div><span>no child</span></div>);
}

---
import { Child } from './child'

export function App() {
  return (<div><Child /></div>);
}
