import { useState, useLayoutEffect } from 'preact/hooks'

export function App() {
  const [theme, setTheme] = useState('light');
  useLayoutEffect(() => {
    setTimeout(() => {
      setTheme('dark');
    }, 100);
  }, []);
  return (<div><span>before: {theme}</span></div>);
}

---
import { useState, useLayoutEffect } from 'preact/hooks'

export function App() {
  const [theme] = useState('light');
  return (<div><span>after: {theme}</span></div>);
}

