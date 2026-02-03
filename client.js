
// const socket = new WebSocket("ws://localhost:3000/ws");
// const input = document.getElementById("msj");
// const button = document.getElementById("btn-enviar");
// let nombreUsuario=prompt("nombre: ")

// button.onclick = function() {
//     const txt=input.value;
//     const message={
//         name:nombreUsuario,
//         msg:txt
//     };
//     socket.send(JSON.stringify(message));
//     console.log("Sent Message!");
//     input.value=""; // limpio el input
// }


// socket.onmessage = function(event){
//     // localizo el tablero
//     const list=document.getElementById("chat-messages");
//     // de json a js object
//     const data=JSON.parse(event.data);
//     // creamos el mensaje
//     const newMessage=document.createElement("li");
//     // ecribir lo que llego del servidor 
//     newMessage.textContent=data.name + ": " + data.msg;
//     // pegamos el mensaje en el tablero
//     list.appendChild(newMessage);
// }





// client.js

// const socket = new WebSocket("ws://localhost:3000/ws");

// // Elementos del DOM
// const loginScreen = document.getElementById("login-screen");
// const chatScreen = document.getElementById("chat-screen");

// const nameInput = document.getElementById("name-input");
// const btnLogin = document.getElementById("btn-login");

// const msgInput = document.getElementById("msj");
// const btnEnviar = document.getElementById("btn-enviar");
// const chatList = document.getElementById("chat-messages");

// let nombreUsuario = null;

// btnLogin.onclick = () => {
//     const nombre = nameInput.value.trim(); 

//     if (nombre) {
//         nombreUsuario = nombre;
//         // Ocultamos login, mostramos chat
//         loginScreen.style.display = "none";     
//         chatScreen.style.display = "flex";      
//         chatScreen.classList.remove("hidden"); 
//     } else {
//         alert("Por favor, escribe un nombre para entrar.");
//     }
// };

// btnEnviar.onclick = () => {
//     if (!nombreUsuario) return; 
    
//     const txt = msgInput.value;
//     if (!txt) return;

//     const message = {
//         name: nombreUsuario,
//         msg: txt
//     };

//     socket.send(JSON.stringify(message));
//     msgInput.value = "";
// };

// socket.onmessage = (event) => {
//     const data = JSON.parse(event.data);
//     const li = document.createElement("li");
    
//     li.innerHTML = `<strong>${data.name}:</strong> ${data.msg}`;
    
//     chatList.appendChild(li);
    
//     chatList.scrollTop = chatList.scrollHeight; 
// };



const path = window.location.pathname;

// LOGICA PAGINA LOGIN
if (path === "/" || path === "/index.html") {
    const btnLogin = document.getElementById("btn-login");
    const nameInput = document.getElementById("name-input");

    if (btnLogin) {
        // entrar con Enter
        nameInput.addEventListener("keypress", (e) => {
            if (e.key === "Enter") btnLogin.click();
        });

        btnLogin.onclick = () => {
            const nombre = nameInput.value.trim();
            if (nombre) {
                sessionStorage.setItem("usuario_chat", nombre);
                window.location.href = "/chat";
            } else {
                alert("Please identify yourself, user.");
            }
        };
    }
}

// LOGICA PAGINA CHAT 
if (path === "/chat") {
    // verificamos si tiene nombre
    const nombreUsuario = sessionStorage.getItem("usuario_chat");

    if (!nombreUsuario) {
        window.location.href = "/";
    } else {
        iniciarChat(nombreUsuario);
    }
}

function iniciarChat(usuario) {
    const headerTitle = document.getElementById("welcome-msg");
    if(headerTitle) headerTitle.innerText = `USER: ${usuario}`;

    const socket = new WebSocket("ws://localhost:3000/ws");
    const msgInput = document.getElementById("msj");
    const btnEnviar = document.getElementById("btn-enviar");
    const chatList = document.getElementById("chat-messages");

    const enviarMensaje = () => {
        const txt = msgInput.value;
        if (!txt) return;

        const message = { name: usuario, msg: txt };
        socket.send(JSON.stringify(message));
        msgInput.value = "";
    };

    btnEnviar.onclick = enviarMensaje;

    msgInput.addEventListener("keypress", (e) => {
        if (e.key === "Enter") enviarMensaje();
    });

    socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        const li = document.createElement("li");
        
        // estilo diferente si soy yo
        if (data.name === usuario) {
            li.classList.add("own-message");
        }

        li.innerHTML = `<strong>${data.name}:</strong> ${data.msg}`;
        chatList.appendChild(li);
        chatList.scrollTop = chatList.scrollHeight;
    };
}