<script lang="ts">
    import {game_status} from "$lib/state.svelte";
    import Emulator from "$lib/Emulator.svelte";
    import Menu from "$lib/Menu.svelte";
    import Welcome from "$lib/Welcome.svelte";
    import {invoke} from "@tauri-apps/api/core";
    import { start_sound, stop_sound } from "$lib/audio";

    const KEY_MAP: {[key: string]: number} = {
        "1": 0x1, "2": 0x2, "3": 0x3, "4": 0xC,
        "q": 0x4, "w": 0x5, "e": 0x6, "r": 0xD,
        "a": 0x7, "s": 0x8, "d": 0x9, "f": 0xE,
        "z": 0xA, "x": 0x0, "c": 0xB, "v": 0xF
    };

    function handleKey(e: KeyboardEvent, isPressed: boolean) {
        if (e.key === "Escape" && isPressed) {
            if (game_status.view === "running") {
                game_status.view = "paused"
                invoke("send_key_event", {key: 0x10, isPressed});
                stop_sound();
            } else if (game_status.view === "paused") {
                game_status.view = "running"
                invoke("send_key_event", {key: 0x11, isPressed});
                if (game_status.sound_active) {
                    start_sound();
                }
            }
            return;
        }

        // Only send keys to backend if running
        if (game_status.view === "running") {
            const key = KEY_MAP[e.key.toLowerCase()];
            invoke("send_key_event", {key, isPressed});
        }
    }
</script>

<svelte:window
    onkeydown={(e) => handleKey(e, true)}
    onkeyup={(e) => handleKey(e, false)}
/>

<main>
    {#if game_status.view === "welcome"}
        <Welcome />
    {:else}
        {#key game_status.session}
            <Emulator />
        {/key}

        {#if game_status.view === "paused"}
            <Menu />
        {/if}
    {/if}
</main>

<style>
    main {
        width: 100vw;
        height: 100vh;
        display: flex;
        justify-content: center;
        align-items: center;
        background: #000;
        overflow: hidden;
        position: relative;
        font-family: sans-serif;
    }
</style>