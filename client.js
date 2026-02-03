
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

// // Variable para guardar el nombre (inicia vacía)
// let nombreUsuario = null;

// // --- LÓGICA DE LOGIN ---
// btnLogin.onclick = () => {
//     const nombre = nameInput.value.trim(); // .trim() quita espacios vacíos al inicio/final

//     if (nombre) {
//         nombreUsuario = nombre;
//         // Ocultamos login, mostramos chat
//         loginScreen.style.display = "none";     // Opción directa JS
//         chatScreen.style.display = "flex";      // Volvemos a flex para que mantenga el diseño
//         chatScreen.classList.remove("hidden");  // O quitamos la clase hidden
//     } else {
//         alert("Por favor, escribe un nombre para entrar.");
//     }
// };

// // --- LÓGICA DE ENVÍO ---
// btnEnviar.onclick = () => {
//     if (!nombreUsuario) return; // Seguridad extra
    
//     const txt = msgInput.value;
//     if (!txt) return; // No enviar mensajes vacíos

//     const message = {
//         name: nombreUsuario,
//         msg: txt
//     };

//     socket.send(JSON.stringify(message));
//     msgInput.value = "";
// };

// // --- LÓGICA DE RECEPCIÓN ---
// socket.onmessage = (event) => {
//     const data = JSON.parse(event.data);
//     const li = document.createElement("li");
    
//     // Un toque extra: poner el nombre en negrita
//     li.innerHTML = `<strong>${data.name}:</strong> ${data.msg}`;
    
//     chatList.appendChild(li);
    
//     // Auto-scroll hacia abajo cuando llega un mensaje nuevo
//     chatList.scrollTop = chatList.scrollHeight; 
// };



const path = window.location.pathname;

// ==========================================
// LÓGICA DE PÁGINA 1: LOGIN ("/" o "/index.html")
// ==========================================
if (path === "/" || path === "/index.html") {
    const btnLogin = document.getElementById("btn-login");
    const nameInput = document.getElementById("name-input");

    if (btnLogin) {
        // Permitir entrar con tecla Enter
        nameInput.addEventListener("keypress", (e) => {
            if (e.key === "Enter") btnLogin.click();
        });

        btnLogin.onclick = () => {
            const nombre = nameInput.value.trim();
            if (nombre) {
                // Guardamos el nombre en la mochila temporal
                sessionStorage.setItem("usuario_chat", nombre);
                // Saltamos a la pagina de chat
                window.location.href = "/chat";
            } else {
                alert("Please identify yourself, user.");
            }
        };
    }
}

// ==========================================
// LÓGICA DE PÁGINA 2: CHAT ("/chat")
// ==========================================
if (path === "/chat") {
    // 1. Verificamos si tiene permiso (nombre)
    const nombreUsuario = sessionStorage.getItem("usuario_chat");

    if (!nombreUsuario) {
        // Si no hay nombre, patada de vuelta al login
        window.location.href = "/";
    } else {
        // Si hay nombre, iniciamos los motores
        iniciarChat(nombreUsuario);
    }
}

function iniciarChat(usuario) {
    // Actualizar título
    const headerTitle = document.getElementById("welcome-msg");
    if(headerTitle) headerTitle.innerText = `USER: ${usuario}`;

    const socket = new WebSocket("ws://localhost:3000/ws");
    const msgInput = document.getElementById("msj");
    const btnEnviar = document.getElementById("btn-enviar");
    const chatList = document.getElementById("chat-messages");

    // Función enviar
    const enviarMensaje = () => {
        const txt = msgInput.value;
        if (!txt) return;

        const message = { name: usuario, msg: txt };
        socket.send(JSON.stringify(message));
        msgInput.value = "";
    };

    btnEnviar.onclick = enviarMensaje;

    // Enviar con Enter
    msgInput.addEventListener("keypress", (e) => {
        if (e.key === "Enter") enviarMensaje();
    });

    socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        const li = document.createElement("li");
        
        // Estilo diferente si soy yo
        if (data.name === usuario) {
            li.classList.add("own-message"); // (Necesitarás agregar esta clase en CSS si quieres)
        }

        li.innerHTML = `<strong>${data.name}:</strong> ${data.msg}`;
        chatList.appendChild(li);
        chatList.scrollTop = chatList.scrollHeight;
    };
}