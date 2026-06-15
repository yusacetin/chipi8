<script lang="ts">
    import {game_status, settings} from "$lib/state.svelte.js";
    import {open} from "@tauri-apps/plugin-dialog";
    import {invoke} from "@tauri-apps/api/core";
    import {getCurrentWindow, LogicalSize} from "@tauri-apps/api/window";
    import {start_sound, stop_sound} from "$lib/audio.ts";

    async function unpause() {
        await invoke("send_key_event", {key: 0x11, isPressed: true});
        game_status.view = "running";
        if (game_status.sound_active) {
            start_sound();
        }
    }

    async function updateSpeed() {
        let encodedSpeed = settings.speed + 0x20;
        await invoke("send_key_event", {key: encodedSpeed, isPressed: true});
    }

    async function updateScale() {
        if (settings.scale < 10) {
            settings.scale = 10;
        }
        const width = settings.width * settings.scale;
        const height = settings.height * settings.scale;
        await getCurrentWindow().setSize(new LogicalSize(width, height));
    }

    async function changeRom() {
        const choice = await open({
            multiple: false,
            filters: [{
                name: "CHIP-8 ROM",
                extensions: ["ch8"]
            }]
        });

        if (choice && typeof choice === "string") {
            game_status.rom_fpath = choice;
            game_status.session++;
            game_status.view = "running";
        }
    }
</script>

<div class="overlay">
    <div class="menu-card">
        <h1>Paused</h1>

        <div class="row">
            <span>Speed: </span> 
            <input type="number" min="1" max="50" bind:value={settings.speed} onchange="{updateSpeed}"/>
        </div>

        <div class="row">
            <span>Scale: </span>
            <input type="number" min="10" max="32" bind:value={settings.scale} onchange="{updateScale}"/>
        </div>

        <div class="row">
            <button onclick={unpause}>Resume</button>
        </div>

        <div class="row">
            <button onclick={changeRom}>Change ROM</button>
        </div>
    </div>
</div>

<style>
    .overlay {
        position: absolute;
        inset: 0;
        background: rgba(0, 0, 0, 0.7);
        backdrop-filter: blur(8px);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 10;
    }

    .menu-card {
        background: #222;
        padding: 2rem;
        border-radius: 8px;
        color: white;
        text-align: center;
    }

    .row {
        margin: 16px;
    }

    button {
        font-size: 20px;
        font-family: "Noto Sans", sans-serif;
        border-radius: 9px;
        background-color: #2D2D2F;
        border: #525254 solid 2px;
        padding-top: 2px;
        padding-bottom: 2px;
        padding-right: 10px;
        padding-left: 10px;
        color: #FAFAFA;
        margin: 0.1em;
    }

    button:hover {
        background-color: #444444;
    }

    button:active {
        background-color: #626262;
    }

    h1 {
        font-size: 36px;
    }

    input[type="number"] {
        font-size: 16px;
        border-radius: 8px;
        padding: 2px 6px;
        margin-left: 5px;
        outline: none;
        border: #525254 solid 2px;
        background-color: #28282A;
        color: #FAFAFA;
        font-family: "Noto Sans", sans-serif;
    }

    input[type="number"]:focus {
        border: rgb(25, 118, 210) solid 2px;
    }

    span {
        font-size: 20px;
    }
</style>