import { useEffect, useState } from "react";
import axios from "axios";

const API_URL = "http://127.0.0.1:3000"; // Your Rust backend

interface Message {
  username: string;
  content: string;
}

function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [username, setUsername] = useState("");
  const [message, setMessage] = useState("");
  const [ws, setWs] = useState<WebSocket | null>(null);

  // Fetch existing messages on load
  useEffect(() => {
    axios.get<Message[]>(`${API_URL}/messages`).then((res) => {
      setMessages(res.data);
    });

    // Connect to WebSocket for real-time updates
    const socket = new WebSocket(`ws://127.0.0.1:3000/ws`);
    socket.onmessage = (event) => {
      const newMessage: Message = JSON.parse(event.data);
      setMessages((prev) => [...prev, newMessage]);
    };

    setWs(socket);

    return () => socket.close();
  }, []);

  // Send message to backend
  const sendMessage = async () => {
    if (!username || !message) return;

    await axios.post(`${API_URL}/send`, { username, content: message });
    setMessage(""); // Clear input after sending
  };

  return (
    <div style={{ maxWidth: "500px", margin: "auto", textAlign: "center" }}>
      <h1>Mr-9 Chat App</h1>
      <input
        type="text"
        placeholder="Username"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        style={{ width: "100%", marginBottom: "8px", padding: "8px" }}
      />
      <input
        type="text"
        placeholder="Type a message..."
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        style={{ width: "100%", marginBottom: "8px", padding: "8px" }}
      />
      <button onClick={sendMessage} style={{ padding: "8px 16px" }}>
        Send
      </button>
      <ul style={{ listStyle: "none", padding: "0", marginTop: "16px" }}>
        {messages.map((msg, index) => (
          <li key={index} style={{ padding: "8px", borderBottom: "1px solid #ccc" }}>
            <strong>{msg.username}:</strong> {msg.content}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
