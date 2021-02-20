import init, * as nes from '../pkg/nes_wasm.js';

function openRom(event) {
    var input = event.target;
    
    var reader = new FileReader();
    reader.onload = function(){
        var arrayBuffer = reader.result;

        nes.load_rom(new Uint8Array(arrayBuffer));
    };
    reader.readAsArrayBuffer(input.files[0]);
}

async function run() {
    await init();

    let file_selector = document.getElementById("rom_input");
    file_selector.addEventListener("change", openRom);
}

run();