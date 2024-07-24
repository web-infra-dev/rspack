import { createContext } from 'preact/compat'
import { useContext } from 'preact/hooks'

const Theme = createContext('light')

function Inner() {
  const theme = useContext(Theme)
  return (<div><span>before: {theme}</span></div>)
}

export function App() {
  return (
    <Theme.Provider value={"dark"}>
      <Inner />
    </Theme.Provider>
  )
}

---
import { createContext } from 'preact/compat'
import { useContext } from 'preact/hooks'

const Theme = createContext('light')

function Inner() {
  const theme = useContext(Theme)
  return (<div><span>after: {theme}</span></div>)
}

export function App() {
  return (
    <Theme.Provider value={"light"}>
      <Inner />
    </Theme.Provider>
  )
}

