import { RTMPServer, RTMPConnection } from "./server.js";
const server = new RTMPServer(1935);
console.log(server);
setInterval(() => {
    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
      console.log('WebSocket connected');
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onmessage = (event) => {
    console.log(event)        
    };
    setTimeout(() => {
        ws.close()
        }, 5000);
}, 30000);