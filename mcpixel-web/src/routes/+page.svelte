<script lang="ts">
    import {PixelArt, Version} from "mcpixel";
    import bert from "$lib/assets/bert.png?uint8array";
    import versionData from "../../../data/1.21.11?uint8array";
    import {onMount} from "svelte";

    // todo: version support
    onMount(async () => {
        const version = new Version(versionData);
        const art = new PixelArt(bert, version);
        console.log(art.blocks())
    });
</script>
