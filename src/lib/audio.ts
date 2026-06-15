const AUDIO_FREQ = 440;
const AUDIO_LEVEL = 0.05;

let audio_ctx = new AudioContext();
let oscillator: OscillatorNode | null = null;
let gain_node: GainNode | null = null;

export async function initialize_audio() {
    if (!audio_ctx) {
        audio_ctx = new AudioContext();
    }
    if (audio_ctx.state !== "running") {
        await audio_ctx.resume();
    }
}

export async function start_sound() {
    if (oscillator) {
        // Already playing
        return;
    }

    if (!audio_ctx) {
        audio_ctx = new AudioContext();
    }

    gain_node = audio_ctx.createGain();
    gain_node.gain.setValueAtTime(0, audio_ctx.currentTime);
    gain_node.gain.linearRampToValueAtTime(AUDIO_LEVEL, audio_ctx.currentTime + 0.01);

    oscillator = audio_ctx.createOscillator();
    oscillator.type = "square";
    oscillator.frequency.setValueAtTime(AUDIO_FREQ, audio_ctx.currentTime);
    oscillator.connect(gain_node);
    gain_node.connect(audio_ctx.destination);
    oscillator.start();
}

export function stop_sound() {
    if (!oscillator || !gain_node || !audio_ctx) {
        return;
    }

    const stop_time = audio_ctx.currentTime + 0.01;
    gain_node.gain.setValueAtTime(gain_node.gain.value, audio_ctx.currentTime);
    gain_node.gain.linearRampToValueAtTime(0, stop_time);
    oscillator.stop(stop_time);

    const old_osc = oscillator;
    const old_gain_node = gain_node;
    oscillator = null;
    gain_node = null;

    setTimeout(() => {
        old_osc.disconnect();
        old_gain_node.disconnect();
    }, 60);
}