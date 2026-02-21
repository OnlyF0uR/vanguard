const connectBtn = document.getElementById('ws-connect');
const disconnectBtn = document.getElementById('ws-disconnect');
const pingBtn = document.getElementById('ws-ping');
const incBtn = document.getElementById('ws-inc');
const logs = document.getElementById('ws-logs');

let ws = null;

function logMsg(msg, color = '#a9b1d6') {
    const div = document.createElement('div');
    div.style.color = color;
    div.innerText = `> ${msg}`;
    logs.appendChild(div);
    logs.scrollTop = logs.scrollHeight;
}

connectBtn.onclick = () => {
    if (ws) return;
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${location.host}/ws/stream`);

    ws.onopen = () => {
        logMsg('Connected to WebSocket server', '#9ece6a');
        connectBtn.disabled = true;
        disconnectBtn.disabled = false;
        pingBtn.disabled = false;
        incBtn.disabled = false;
    };

    ws.onmessage = (e) => {
        try {
            const data = JSON.parse(e.data);
            logMsg(`Rx [${data.topic}]: ${data.data}`, '#7aa2f7');

            if (data.topic === 'counter_update' && typeof window.Router !== 'undefined') {
                const newCount = parseInt(data.data, 10);
                if (!isNaN(newCount)) {
                    window.Router.setGlobalState('counter', { count: newCount }, true);
                }
            }
        } catch {
            logMsg(`Rx: ${e.data}`);
        }
    };

    ws.onclose = () => {
        logMsg('Disconnected', '#f7768e');
        ws = null;
        connectBtn.disabled = false;
        disconnectBtn.disabled = true;
        pingBtn.disabled = true;
        incBtn.disabled = true;
    };
};

disconnectBtn.onclick = () => {
    if (ws) ws.close();
};

pingBtn.onclick = () => {
    if (ws) {
        ws.send(JSON.stringify({ action: 'ping' }));
        logMsg('Tx: ping', '#e0af68');
    }
};

incBtn.onclick = () => {
    if (ws) {
        ws.send(JSON.stringify({ action: 'increment_counter' }));
        logMsg('Tx: increment_counter', '#e0af68');
    }
};

if (typeof window.Router !== 'undefined' && window.Router.onCleanup) {
    window.Router.onCleanup(() => {
        if (ws) ws.close();
    });
}
