<script>
    /**
     * @typedef {Object} Props
     * @property {number} [opacity]
     * @property {number} [width]
     */

    /** @type {Props} */
    let { opacity = 0.9, width = 24 } = $props();
</script>

<svg
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width={2}
        width={width}
        opacity={opacity}
>
    <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
    />
</svg>
