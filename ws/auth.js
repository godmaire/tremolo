const socket = new WebSocket("ws://localhost:8000/ws/agent");

socket.addEventListener("open", (event) => {
    const msg = JSON.stringify({
        "AuthRequest": {
            "name": "Fake Client",
            "token": "fake-token",
        },
    });
    socket.send(msg);
});

socket.addEventListener("message", (event) => {
    console.log(event.data)
})

socket.addEventListener("close", (event) => {
    console.log("Socket closed.")
})
