<script>
  import { createEventDispatcher } from "svelte";

  export let id = "";
  export let name = "";
  export let checked = false;
  export let disabled = false;
  export let required = false;
  export let value = undefined;
  export let ariaLabel = "";
  export let className = "";

  const dispatch = createEventDispatcher();

  const handleChange = (event) => {
    dispatch("change", event);
  };

  const handleInput = (event) => {
    dispatch("input", event);
  };
</script>

<div class={`content ${className}`.trim()}>
  <label class="checkBox" class:is-disabled={disabled}>
    <input
      id={id}
      name={name}
      type="checkbox"
      bind:checked
      disabled={disabled}
      required={required}
      value={value}
      aria-label={ariaLabel}
      on:change={handleChange}
      on:input={handleInput}
    />
    <div class="transition"></div>
  </label>
</div>

<style>
  .content {
    display: inline-flex;
  }

  .checkBox {
    display: block;
    cursor: pointer;
    width: 30px;
    height: 30px;
    border: 3px solid rgba(255, 255, 255, 0);
    border-radius: 10px;
    position: relative;
    overflow: hidden;
    box-shadow: 0px 0px 0px 2px #fff;
  }

  .checkBox div {
    width: 60px;
    height: 60px;
    background-color: #fff;
    top: -52px;
    left: -52px;
    position: absolute;
    transform: rotateZ(45deg);
    z-index: 100;
  }

  .checkBox input[type="checkbox"]:checked + div {
    left: -10px;
    top: -10px;
  }

  .checkBox input[type="checkbox"] {
    position: absolute;
    left: 50px;
    visibility: hidden;
  }

  .transition {
    transition: 300ms ease;
  }

  .is-disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
