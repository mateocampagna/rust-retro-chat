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

    let userColorClass = sessionStorage.getItem("user_color_class");
    if (!userColorClass) {
        const randomNum = Math.floor(Math.random() * 8) + 1;
        userColorClass = `user-color-${randomNum}`;
        sessionStorage.setItem("user_color_class", userColorClass);
    }


    const socket = new WebSocket("ws://localhost:3000/ws");
    const msgInput = document.getElementById("msj");
    const btnEnviar = document.getElementById("btn-enviar");
    const chatList = document.getElementById("chat-messages");

    const enviarMensaje = () => {
        const txt = msgInput.value;
        if (!txt) return;

        const message = { 
            name: usuario, 
            msg: txt,
            color: userColorClass
        };

        socket.send(JSON.stringify(message));
        msgInput.value = "";
    };

    btnEnviar.onclick = enviarMensaje;

    msgInput.addEventListener("keypress", (e) => {
        // Si aprieta Enter Y NO está apretando Shift
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault(); // Evitamos que haga el salto de línea en la caja
            enviarMensaje();
        }
    });;

    socket.onmessage = (event) => {
        const data = JSON.parse(event.data);
        const li = document.createElement("li");
        const timeString = data.time || new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        
        if (data.name === usuario) {
            li.classList.add("own-message"); 
        }
        // Usar parseMarkdown para procesar el contenido
        const messageContent = parseMarkdown(data.msg);
        const colorClass = data.color || 'user-color-2';
        li.innerHTML = `<span class="timestamp">[${timeString}]</span> <strong class="${colorClass}">${data.name}:</strong> <div class="md-content">${messageContent}</div>`;        
        chatList.appendChild(li);
        chatList.scrollTop = chatList.scrollHeight;
    };
}

let katexConfigured = false;

function parseMarkdown(text) {
    // Usar marked.js para parsear markdown
    if (typeof marked !== 'undefined') {
        
        // Conectar KaTeX (Matemáticas) solo la primera vez
        if (typeof markedKatex !== 'undefined' && !katexConfigured) {
            // throwOnError: false evita que el chat se rompa si escribes mal una fórmula
            marked.use(markedKatex({ throwOnError: false }));
            katexConfigured = true;
        }

        const html = marked.parse(text, {
            breaks: true,
            gfm: true // GitHub Flavored Markdown
        });
        
        // Resaltar sintaxis en bloques de código
        if (typeof hljs !== 'undefined') {
            const div = document.createElement('div');
            div.innerHTML = html;
            div.querySelectorAll('pre code').forEach((block) => {
                hljs.highlightElement(block);
            });
            return div.innerHTML;
        }
        
        return html;
    }
    
    // Fallback de seguridad si marked.js no está disponible
    return escapeHtml(text)
        .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
        .replace(/\*(.*?)\*/g, '<em>$1</em>')
        .replace(/`(.*?)`/g, '<code>$1</code>')
        .replace(/\n/g, '<br>');
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}