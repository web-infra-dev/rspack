const def = (o) => {
  o.readFile = () => 42
}

def(exports)
