export const settings = $state({
    scale: 16,
    width: 64,
    height: 32,
    background: "#0A0A0A",
    foreground: "#FAFAFA",
    speed: 2
});

export const game_status = $state({
    view: "welcome" as "welcome" | "running" | "paused",
    rom_loaded: false,
    rom_fpath: "",
    session: 0
});