import { defineComponent, ref } from 'vue'

export default defineComponent({
  setup() {
    const count = ref<number>(0)

    function add() {
      count.value++
    }

    return { count, add }
  },

  render() {
    return (
      <button onClick={this.add}>
        <span>TSX_Counter: </span>
        <strong>{this.count}</strong>
      </button>
    )
  },
})
