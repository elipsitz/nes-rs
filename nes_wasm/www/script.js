import init, * as nes from '../pkg/nes_wasm.js';

const NES_WIDTH = 256;
const NES_HEIGHT = 240;

var emulator = null;
var canvas = null;
var canvas_ctx = null;
var canvas_data = null;
var paused = true;

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
    emulator.emulate_frame();
    emulator.get_frame_buffer(canvas_data.data);
    canvas_ctx.putImageData(canvas_data, 0, 0);

    if (!paused) {
        window.requestAnimationFrame(emulatorStep);
    }
}

async function onLoad() {
    // Initialize wasm.
    await init();

    // Initialize canvas.
    canvas = document.getElementById("canvas");
    canvas_ctx = canvas.getContext("2d");
    canvas_data = canvas_ctx.createImageData(NES_WIDTH, NES_HEIGHT);

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