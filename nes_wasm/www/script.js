import init, * as nes from '../pkg/nes_wasm.js';

const NES_WIDTH = 256;
const NES_HEIGHT = 240;

function openRom(event) {
    var input = event.target;
    
    var reader = new FileReader();
    reader.onload = function(){
        var arrayBuffer = reader.result;
        var data = new Uint8Array(arrayBuffer);
        var emulator = new nes.Emulator(data);
        for (var i = 0; i < 60; i += 1) {
            emulator.emulate_frame();
        }

        var canvas = document.getElementById("canvas");
        var ctx = canvas.getContext("2d");
        var data = ctx.createImageData(NES_WIDTH, NES_HEIGHT);
        emulator.get_frame_buffer(data.data);
        ctx.putImageData(data, 0, 0);
    };
    reader.readAsArrayBuffer(input.files[0]);
}

async function run() {
    await init();

    let file_selector = document.getElementById("rom_input");
    file_selector.addEventListener("change", openRom);
}

run();