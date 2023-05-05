import * as wasm from "crdt";

let textboxText1 = wasm.TextBoxSynchronizer.new(0)

let textboxText2 = wasm.TextBoxSynchronizer.new(1)

let textbox1 = document.getElementById('area1')
let textbox2 = document.getElementById('area2')



function applyOp(text, textbox, op) {
    text.apply_remote_operation(op)
    textbox.value = text.get_text();
    let cursorPos = text.get_absolute_cursor_pos();
    textbox.setSelectionRange(cursorPos, cursorPos)
}



textbox1.addEventListener('keydown', (event) => {
    textboxText1.set_absolute_cursor_pos(textbox1.selectionStart)

    let op
    if (event.key == 'Enter') {
        op =  textboxText1.insert_at_cursor("\n")
    }
    else if (event.key == 'Backspace') {
        if(textbox1.selectionStart == 0) return
        op = textboxText1.remove_at_cursor()
    }
    else if (event.key.length == 1) {
        op =  textboxText1.insert_at_cursor(event.key)
    }
    else return
    applyOp(textboxText2, textbox2, op)
})

textbox2.addEventListener('keydown', (event) => {
    textboxText2.set_absolute_cursor_pos(textbox2.selectionStart)

    let op
    if (event.key == 'Enter') {
        op =  textboxText2.insert_at_cursor("\n")
    }
    else if (event.key == 'Backspace') {
        if(textbox2.selectionStart == 0) return
        op = textboxText2.remove_at_cursor()
    }
    else if (event.key.length == 1) {
        op =  textboxText2.insert_at_cursor(event.key)
    }
    else return
    applyOp(textboxText1, textbox1, op)
})

const domain = "http://localhost:3000"
async function connect() {
    let id = await fetch(`${domain}/register`, {method: 'POST'}).then(v => v.text)
    let socket = new WebSocket(`${domain}/data-stream/${id}`)
    let promise = new Promise((resolve) => socket.onopen = () => resolve(socket))
    return {
        id,
        socket: await promise,
        text: wasm.TextBoxSynchronizer.new(id)
    }
}
