<script>
    import { run } from 'svelte/legacy';

    import Loading from "$lib/Loading.svelte";
    import {onDestroy} from "svelte";

    let { colors, level } = $props();

    let isHover = $state(false);
    let isLoading = $state(false);

    let colBack = $state();
    let colBackHov = $state();
    let colTxt = $state();
    let border = $state();
    let timer = $state();
    let loadCol = $state('white');



    onDestroy(() => {
        clearTimeout(timer);
    });

    function extractColors() {
        switch (level) {
            case 1:
                colBack = colors.act1;
                colBackHov = colors.act1a;
                colTxt = 'white';
                border = `1px solid ${colors.gmid}`;
                break;
            case 2:
                colBack = colors.act2;
                colBackHov = colors.act2a;
                colTxt = colors.ghigh;
                border = `1px solid ${colors.gmid}`;
                break;
            default:
                colBack = colors.ghigh;
                colBackHov = 'white';
                colTxt = colors.act2a;
                border = `1px solid ${colors.act2a}`;
                loadCol = colors.acnt;
        }
    }

    run(() => {
        if (isLoading) {
            timer = setTimeout(() => {
                isLoading = false;
            }, 2000);
        }
    });
    run(() => {
        if (colors) {
            extractColors();
        }
    });
</script>

<div class="wrap">
    <button
            class="btn font-label"
            style:border={border}
            style:width="100px"
            style:box-shadow="1px 1px 2px {colors.gmid}"
            style:cursor="{isLoading ? 'default' : 'pointer'}"
            style:color={colTxt}
            style:background={isHover ? colBackHov : colBack}
            onfocus={() => isHover = true}
            onblur={() => isHover = false}
            onclick={() => isLoading = true}
            onkeypress={() => isLoading = true}
    >
        {#if isLoading}
            <div class="load">
                <Loading background={false} color={loadCol}/>
            </div>
        {:else}
            <div class="txt">
                LOAD
            </div>
        {/if}
    </button>
</div>

<style>
    .btn {
        height: 30px;
        font-weight: bold;
        border-radius: 3px;
        overflow: hidden;
    }

    .load {
        display: flex;
        justify-content: center;
    }

    .txt {
        margin-top: 1px;
    }

    .wrap {
        height: 32px;
        margin: 5px;
    }
</style>
