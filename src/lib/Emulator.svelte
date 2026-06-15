<script lang="ts">
    import {onMount, onDestroy} from "svelte";
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import {settings, game_status} from "$lib/state.svelte";
    import {initialize_audio, start_sound, stop_sound} from "$lib/audio.ts";

    let canvas: HTMLCanvasElement;
    let unlisten: () => void;
    let unlisten_sound: () => void;

    onMount(async () => {
        const ctx = canvas.getContext("2d");
        if (!ctx) {
            return;
        }

        game_status.sound_active = false;
        initialize_audio();

        unlisten = await listen<Uint8Array>("draw-display", (event) => {
            if (game_status.view == "paused") {
                return;
            }

            const screen_data = event.payload;
            ctx.fillStyle = settings.background;
            ctx.fillRect(0, 0, settings.width * settings.scale, settings.height * settings.scale);
            
            ctx.fillStyle = settings.foreground;
            for (let i = 0; i < screen_data.length; i++) {
                if (screen_data[i] === 1) {
                    const x = i % settings.width;
                    const y = Math.floor(i / settings.width);
                    ctx.fillRect(x * settings.scale, y * settings.scale, settings.scale, settings.scale);
                }
            }
        });

        unlisten_sound = await listen<boolean>("play-sound", (event) => {
            game_status.sound_active = event.payload;
            if (game_status.sound_active) {
                start_sound();
            } else {
                stop_sound();
            }
        });

        await invoke("run_emulator", {romFpath: game_status.rom_fpath, speed: settings.speed});
    });

    onDestroy(() => {
        invoke("send_key_event", {key: 0x12, isPressed: true}); // terminate running thread

        if (unlisten) {
            unlisten();
        }
    });
</script>

<canvas bind:this={canvas} width={settings.width * settings.scale} height={settings.height * settings.scale}></canvas>

<style>
    canvas {
        image-rendering: pixelated;
        display: block;
    }

    :global(body) {
        margin: 0;
        padding: 0;
        background-color: #0A0A0A;
        overflow: hidden;
    }
</style>