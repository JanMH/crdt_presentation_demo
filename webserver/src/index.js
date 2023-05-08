const express = require('express');
const ws = require('ws');
const cors = require('cors');


const app = express();


app.use(cors())
app.use(express.static('public'))

// Set up a headless websocket server that prints any
// events that come in.
const wsServer = new ws.Server({ noServer: true });
const messages = []
const connections = []

function syncNewMessages() {
    console.debug("Synchronizing...")
    for (let conn of connections) {
        if(conn == undefined) {
            continue
        }
        if (conn.socket.readyState !== ws.WebSocket.OPEN) {
            console.warn(`Client ${conn.id} isn't ready to accept messages: `, conn.socket.readyState)
        } else {
            syncClient(conn.id)
        }
    }
}

function syncClient(clientId) {
    let offset = connections[clientId].currentOffset
    console.debug("Synchronizing client", clientId, offset)

    if (messages.length <= offset) {
        return
    }
    let msg = JSON.parse(messages[offset])

    connections[clientId].currentOffset++

    if (msg.sent_by == clientId) {
        return
    }
    connections[clientId].socket.send(
        messages[offset],
        (err) => {
            if (err != undefined) {
                console.log(err)
            } else {
                syncClient(clientId)
            }
        }
    )
}

wsServer.on('connection', socket => {
    socket.on('message', message => {
        messages.push(message.toString())
        syncNewMessages()
    });
});

let counter = 0

app.post('/register', (req, res) => {
    res.send(`${counter++}`)
})

const server = app.listen(3000);

server.on('upgrade', (request, socket, head) => {
    wsServer.handleUpgrade(request, socket, head, s => {
        wsServer.emit('connection', s, request);

        let id = Number(request.url.substring(request.url.lastIndexOf("/") + 1))
        connections[id] = { socket: s, currentOffset: 0, id }
        syncClient(id)
    });
});
