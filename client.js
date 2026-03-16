const path = window.location.pathname;

// LOGICA PAGINA LOGIN
// LOGICA PAGINA LOGIN
if (path === "/" || path === "/index.html") {
    const btnLogin = document.getElementById("btn-login");
    const btnRegister = document.getElementById("btn-register");
    const nameInput = document.getElementById("name-input");
    const passwordInput = document.getElementById("password-input");
    const errorMsg = document.getElementById("error-msg");
    
    // Función ROBUSTA para mostrar errores en la UI
    const showError = (text) => {
        errorMsg.innerText = text;
        errorMsg.style.display = "block"; // Forzamos a que se muestre, anulando el CSS inicial
        
        // Ocultar el error cuando el usuario vuelva a escribir
        const hideError = () => {
            errorMsg.style.display = "none"; // Lo volvemos a ocultar
            nameInput.removeEventListener("input", hideError);
            passwordInput.removeEventListener("input", hideError);
        };
        nameInput.addEventListener("input", hideError);
        passwordInput.addEventListener("input", hideError);
    };

    async function authenticate(endpoint) {
        const username = nameInput.value.trim();
        const password = passwordInput.value.trim();

        if (!username || !password) {
            showError("[!] INCOMPLETE CREDENTIALS");
            return;
        }

        try {
            const response = await fetch(endpoint, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password })
            });

            const textData = await response.text();
            let data;
            try {
                data = JSON.parse(textData);
            } catch (e) {
                showError(`[FATAL] SERVER SAID: ${textData.substring(0, 30)}...`);
                return;
            }

            if (response.ok) {
                // FIX 1: Si fue un Registro, NO redirigimos. Avisamos que fue exitoso.
                if (endpoint === "/register") {
                    showError("[✓] REGISTERED SUCCESSFULLY. PLEASE LOGIN.");
                    errorMsg.style.color = "#50fa7b"; // Color Verde
                    errorMsg.style.borderColor = "#50fa7b";
                    passwordInput.value = ""; // Limpiamos la clave
                    return; // ¡Cortamos la función acá!
                }

                // FIX 2: Si fue Login, guardamos Token y ahí sí redirigimos
                const username_lower = username.toLowerCase();
                sessionStorage.setItem("usuario_chat", username_lower);
                
                if (data.token) {
                    sessionStorage.setItem("jwt_token", data.token);
                }

                window.location.href = "/chat";
            } 
            else {
                // Restauramos el color rojo por si venía de un registro exitoso previo
                errorMsg.style.color = "#ff0000";
                errorMsg.style.borderColor = "#ff0000";
                
                showError(`[ACCESS DENIED] ${data.message}`);
                passwordInput.value = ""; // Limpiamos la clave
                passwordInput.focus();
            }
        } catch (error) {
            showError("[FATAL] CONNECTION TO MAINFRAME LOST");
        }
    }

    // Encendido de botones
    if (btnLogin && btnRegister) {
        passwordInput.addEventListener("keypress", (e) => {
            if (e.key === "Enter") authenticate("/login");
        });

        btnLogin.onclick = () => authenticate("/login");
        btnRegister.onclick = () => authenticate("/register");
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

    const jwtToken = sessionStorage.getItem("jwt_token");

    if (!jwtToken) {
        window.location.href = "/";
        return;
    }

    // Le pegamos el token a la URL del WebSocket
    const socket = new WebSocket(`ws://localhost:3000/ws?token=${jwtToken}`);
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