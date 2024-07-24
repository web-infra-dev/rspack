import { createContext } from 'preact/compat'
import { useContext } from 'preact/hooks'

const Theme = createContext('light')

function Inner() {
  const theme = useContext(Theme)
  return (<div><span>before: {theme}</span></div>)
}

export function App() {
  return (
    <Inner />
  )
}

---
import { createContext } from 'preact/compat'
import { useContext } from 'preact/hooks'

const Theme = createContext('blue')

function Inner() {
  const theme = useContext(Theme)
  return (<div><span>after: {theme}</span></div>)
}

export function App() {
  return (
    <Inner />
  )
}

