import init, * as nes from '../pkg/nes_wasm.js';

function openRom(event) {
    var input = event.target;
    
    var reader = new FileReader();
    reader.onload = function(){
        var arrayBuffer = reader.result;
        var data = new Uint8Array(arrayBuffer);
        var emulator = new nes.Emulator(data);
        console.time("frame");
        for (var i = 0; i < 60; i += 1) {
            emulator.emulate_frame();
        }
        console.timeEnd("frame");
    };
    reader.readAsArrayBuffer(input.files[0]);
}

async function run() {
    await init();

    let file_selector = document.getElementById("rom_input");
    file_selector.addEventListener("change", openRom);
}

run();