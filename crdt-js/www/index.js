import * as wasm from "crdt";
const host = 'localhost'
const port = 3000
const baseUrl = `http://${host}:${port}`

let textarea = document.getElementById('area')
connectTextarea(textarea)


function applyOp(text, textbox, op) {
    text.apply_remote_operation(op)
    textbox.value = text.get_text();
    let cursorPos = text.get_absolute_cursor_pos();
    textbox.setSelectionRange(cursorPos, cursorPos)
}

async function connectTextarea(textarea) {
    textarea.value = "Connecting..."
    let synchronizer = await connect()
    textarea.value = ""
    synchronizer.socket.addEventListener('message', (msg) => {
        synchronizer.text.set_absolute_cursor_pos(textarea.selectionStart)
        applyOp(synchronizer.text, textarea, msg.data)
    })


    textarea.addEventListener('keydown', (event) => {
        synchronizer.text.set_absolute_cursor_pos(textarea.selectionStart)

        let op
        if (event.key == 'Enter') {
            op = synchronizer.text.insert_at_cursor("\n")
        }
        else if (event.key == 'Backspace') {
            op = synchronizer.text.remove_at_cursor()
        }
        else if (event.key.length == 1) {
            op = synchronizer.text.insert_at_cursor(event.key)
        }
        else return
        
        if(op == undefined) return

        synchronizer.socket.send(op)
    })
}


async function connect() {
    let id = await (fetch(`/register`, { method: 'POST' }).then(v => v.text()))
    let socket = new WebSocket("ws"+ document.location.origin.substring(4)  + `/data-stream/${id}`)
    let promise = new Promise((resolve) => socket.onopen = () => resolve(socket))
    await promise
    return {
        id,
        socket,
        text: wasm.TextBoxSynchronizer.new(id)
    }
}
