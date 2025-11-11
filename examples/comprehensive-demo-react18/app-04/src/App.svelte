<script>
  export let name;

  window.addEventListener("change-name", e => {
    console.log(e);
    if (typeof e.detail !== "undefined") {
      name = e.detail.name;
    }
  });
</script>

<style>
  h1 {
    color: purple;
  }
</style>

<h1>Hello From Svelte {name}!</h1>
