<script lang="tsx">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  setup() {
    const count = ref<number>(0)

    function add() {
      count.value++
    }

    return () => (
      <button onClick={add}>
        <span>SFC_TSX_Counter: </span>
        <strong>{count.value}</strong>
      </button>
    )
  },
})
</script>
