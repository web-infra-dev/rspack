import logo from './logo.svg';
import styles from './App.module.css';
import { createSignal } from "solid-js";
import Message from './Message';
function App() {
  const [count, setCount] = createSignal(1);
  const increment = () => setCount(count() + 1);
  return (
    <div class={styles.App}>
      <header class={styles.header}>
        <img src={logo} class={styles.logo} alt="logo" />
        <p>
          Edit <code>src/App.jsx</code> and save to reload.
        </p>
        <a
          class={styles.link}
          href="https://github.com/solidjs/solid"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn Solid
        </a>
        <button type="button" onClick={increment}>
          {count()}
        </button>
        <Message/>
      </header>
    </div>
  );
}

export default App;
