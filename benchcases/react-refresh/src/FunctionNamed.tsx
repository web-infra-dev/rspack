import { useEffect, useState } from 'react'

export function FunctionNamed() {
  const [data, setData] = useState(0)

  useEffect(() => {
    setInterval(() => {
      setData((i) => i + 1)
    }, 100)
  }, [])

  return <h1>Named Export Function {data}</h1>
}
