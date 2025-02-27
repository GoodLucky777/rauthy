<script>
    import { run } from 'svelte/legacy';

    import {onMount} from "svelte";
    import IconBarsArrowDown from "$lib/icons/IconBarsArrowDown.svelte";
    import IconBarsArrowUp from "$lib/icons/IconBarsArrowUp.svelte";
    import Tooltip from "../Tooltip.svelte";

    /**
     * @typedef {Object} Props
     * @property {any} [items]
     * @property {any} resItems
     * @property {any} [options]
     * @property {boolean} [firstDirReverse]
     */

    /** @type {Props} */
    let {
        items = [],
        resItems = $bindable(),
        options = [],
        firstDirReverse = false
    } = $props();

    let selected = $state('');
    let direction = $state(1);
    let callback;

    onMount(() => {
        if (options.length > 0) {
            selected = options[0].label;
            extractCallback();
        }
        if (firstDirReverse) {
            switchDirection();
        }
        orderItems();
    });



    function extractCallback() {
        for (let opt of options) {
            if (opt.label === selected) {
                callback = opt.callback;
                break;
            }
        }

        if (!callback) {
            console.error('Could not find a valid callback function in order options for label ' + selected);
        }
    }

    function orderItems() {
        if (callback) {
            let sorted = [...items];
            sorted.sort((a, b) => {
                return callback(a, b) * direction;
            });
            resItems = [...sorted];
        }
    }

    function switchDirection() {
        direction *= -1;
        orderItems();
    }
    run(() => {
        if (items) {
            orderItems();
        }
    });
    run(() => {
        if (selected) {
            extractCallback();
            orderItems();
        }
    });
</script>

<div class="container">
    {#if options.length > 1}
        <Tooltip text="Order by" yOffset={-30}>
            <select class="opts font-label" bind:value={selected}>
                {#each options as opt}
                    <option value={opt.label}>
                        {opt.label}
                    </option>
                {/each}
            </select>
        </Tooltip>
    {/if}

    {#if options.length > 0}
        <div
                role="button"
                tabindex="0"
                class="icon"
                onclick={switchDirection}
                onkeypress={switchDirection}
        >
            {#if direction === 1}
                <IconBarsArrowUp/>
            {:else}
                <IconBarsArrowDown/>
            {/if}
        </div>
    {/if}
</div>

<style>
    .opts {
        margin-right: 15px;
    }

    .container {
        width: 100%;
        display: flex;
        align-items: center;
    }

    .icon {
        margin-top: 5px;
        cursor: pointer;
        color: var(--col-act2a)
    }

    select {
        height: 2.13rem;
        padding-top: .2rem;
        padding-left: .5rem;
        color: var(--col-text);
        background: var(--col-bg);
        font-size: 1.05rem;
        border-radius: 3px;
        cursor: pointer;
        border: 1px solid var(--col-glow);
        box-shadow: 1px 1px 2px var(--col-gmid);
    }
</style>
