<script lang="jsx">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  setup() {
    const count = ref(0)

    function add() {
      count.value++
    }

    return () => (
      <button onClick={add}>
        <span>SFC_JSX_Counter: </span>
        <strong>{count.value}</strong>
      </button>
    )
  },
})
</script>
