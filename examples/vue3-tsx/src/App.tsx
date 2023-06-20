import { FunctionalComponent, defineComponent, ref } from "vue";
import vueLogo from "./assets/vue.svg";
import "./App.css";

const FcNode: FunctionalComponent = () => {
  return <>123123</>;
};

export default defineComponent({
  name: "App",

  setup() {
    const count = ref(0);
    const add = () => count.value++;

    return () => (
      <div class="App">
        <h1>Hello world!</h1>
        <div>
          <a href="https://vuejs.org" target="_blank">
            <img src={vueLogo} class="logo vue" alt="Vue logo" />
          </a>
        </div>
        <h1>Rspack + Vue TSX</h1>
        <FcNode />
        <div class="card">
          <button onClick={add}>count is {count.value}</button>
        </div>
      </div>
    );
  },
});
