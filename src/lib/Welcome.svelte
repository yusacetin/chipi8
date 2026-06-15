<script lang="ts">
    import {onMount} from "svelte";
    import {game_status, settings} from "$lib/state.svelte";
    import {open} from "@tauri-apps/plugin-dialog";
    import {getCurrentWindow, LogicalSize} from "@tauri-apps/api/window";

    async function chooseRom() {
        const choice = await open({
            multiple: false,
            filters: [{
                name: "CHIP-8 ROM",
                extensions: ["ch8"]
            }]
        });

        if (choice && typeof choice === "string") {
            game_status.rom_fpath = choice;
            game_status.view = "running";
        }
    }

    async function updateScale() {
        if (settings.scale < 10) {
            settings.scale = 10;
        }
        const width = settings.width * settings.scale;
        const height = settings.height * settings.scale;
        await getCurrentWindow().setSize(new LogicalSize(width, height));
    }

    // Fixes window sizing issue on GNOME desktop
    onMount(() => {
        updateScale();
    });
</script>

<div class="overlay">
    <div class="menu-card">
        <h1>Chipi8 CHIP-8 Emulator</h1>

        <div class="row">
            <span>Speed: </span>
            <input type="number" min="1" max="50" bind:value={settings.speed}/>
        </div>

        <div class="row">
            <span>Scale: </span>
            <input type="number" min="10" max="32" bind:value={settings.scale} onchange="{updateScale}"/>
        </div>

        <div class="row">
            <button onclick={chooseRom}>Choose ROM</button>
        </div>
    </div>
</div>

<style>
    .overlay {
        position: absolute;
        inset: 0;
        background: #0A0A0A;
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 10;
    }

    .menu-card {
        background: #181818;
        padding: 20px;
        padding-right: 60px;
        padding-left: 60px;
        border-radius: 16px;
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