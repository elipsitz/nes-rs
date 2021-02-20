import init, * as nes from '../pkg/nes_wasm.js';

const NES_WIDTH = 256;
const NES_HEIGHT = 240;
const NES_SAMPLE_RATE = 48000;

var emulator = null;
var canvas = null;
var canvas_ctx = null;
var canvas_data = null;
var audio_ctx = null;
var audio_buffer = null;
var paused = true;
var key_a = false;
var key_b = false;
var key_select = false;
var key_start = false;
var key_up = false;
var key_down = false;
var key_left = false;
var key_right = false;

function openRom(event) {
    var input = event.target;
    
    var reader = new FileReader();
    reader.onload = function(){
        var arrayBuffer = reader.result;
        var data = new Uint8Array(arrayBuffer);
        emulator = new nes.Emulator(data);

        if (paused) {
            paused = false;
            emulatorStep();
        }
    };
    reader.readAsArrayBuffer(input.files[0]);
}

function emulatorStep() {
    emulator.set_controller1_state(
        key_a,
        key_b,
        key_select,
        key_start,
        key_left,
        key_right,
        key_up,
        key_down,
    );
    emulator.emulate_frame();
    emulator.get_frame_buffer(canvas_data.data);
    canvas_ctx.putImageData(canvas_data, 0, 0);

    emulator.get_audio_buffer(audio_buffer.getChannelData(0));
    var source = audio_ctx.createBufferSource();
    source.buffer = audio_buffer;
    source.connect(audio_ctx.destination);
    source.start();

    if (!paused) {
        window.requestAnimationFrame(emulatorStep);
    }
}

function handleKey(code, down) {
    switch (code) {
        case "KeyZ":
            key_a = down;
            break;
        case "KeyX":
            key_b = down;
            break;
        case "ShiftRight":
            key_select = down;
            break;
        case "Enter":
            key_start = down;
            break;
        case "ArrowUp":
            key_up = down;
            break;
        case "ArrowDown":
            key_down = down;
            break;
        case "ArrowLeft":
            key_left = down;
            break;
        case "ArrowRight":
            key_right = down;
            break;
    }
}

async function onLoad() {
    // Initialize wasm.
    await init();

    // Initialize canvas.
    canvas = document.getElementById("canvas");
    canvas_ctx = canvas.getContext("2d");
    canvas_data = canvas_ctx.createImageData(NES_WIDTH, NES_HEIGHT);
    canvas.addEventListener("keydown", (e) => handleKey(e.code, true), true);
    canvas.addEventListener("keyup", (e) => handleKey(e.code, false), true);

    // Initialize audio.
    var AudioContext = window.AudioContext || window.webkitAudioContext;
    audio_ctx = new AudioContext({sampleRate: NES_SAMPLE_RATE});
    audio_buffer = audio_ctx.createBuffer(1, NES_SAMPLE_RATE / 60, NES_SAMPLE_RATE);

    // Initialize controls;
    let file_selector = document.getElementById("rom_input");
    file_selector.addEventListener("change", openRom);
    document.getElementById("control_pause").addEventListener("click", () => {
        paused = true;
    });
    document.getElementById("control_play").addEventListener("click", () => {
        if (paused) {
            paused = false;
            emulatorStep();
        }
    });
    document.getElementById("control_step").addEventListener("click", () => {
        paused = true;
        emulatorStep();
    });
}

window.addEventListener("load", onLoad);