import { useEffect, useState } from "react";
import axios from "axios";

const API_URL = "http://127.0.0.1:3000"; // Your Rust backend

interface Message {
  username: string;
  content: string;
  is_ai?: boolean;
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

    await axios.post(`${API_URL}/send`, { 
      username, 
      content: message,
      is_ai: false
    });
    setMessage(""); // Clear input after sending
  };

  return (
    <div style={{ maxWidth: "800px", margin: "auto", padding: "20px" }}>
      <h1 style={{ textAlign: "center", marginBottom: "30px" }}>Mr-9 Chat App with Claude AI</h1>
      <div style={{ marginBottom: "20px" }}>
        <input
          type="text"
          placeholder="Your Name"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          style={{ 
            width: "100%", 
            padding: "10px",
            marginBottom: "10px",
            borderRadius: "5px",
            border: "1px solid #ccc"
          }}
        />
        <div style={{ display: "flex", gap: "10px" }}>
          <input
            type="text"
            placeholder="Type a message..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={(e) => e.key === "Enter" && sendMessage()}
            style={{ 
              flex: 1,
              padding: "10px",
              borderRadius: "5px",
              border: "1px solid #ccc"
            }}
          />
          <button 
            onClick={sendMessage}
            style={{
              padding: "10px 20px",
              borderRadius: "5px",
              border: "none",
              backgroundColor: "#0066cc",
              color: "white",
              cursor: "pointer"
            }}
          >
            Send
          </button>
        </div>
      </div>
      <div style={{ 
        border: "1px solid #ccc",
        borderRadius: "5px",
        height: "500px",
        overflowY: "auto",
        padding: "20px"
      }}>
        {messages.map((msg, index) => (
          <div 
            key={index} 
            style={{ 
              marginBottom: "15px",
              padding: "10px",
              borderRadius: "5px",
              backgroundColor: msg.is_ai ? "#f0f7ff" : "#ffffff",
              border: "1px solid #e0e0e0"
            }}
          >
            <div style={{ 
              fontWeight: "bold", 
              color: msg.is_ai ? "#0066cc" : "#333",
              marginBottom: "5px"
            }}>
              {msg.username}
            </div>
            <div style={{ whiteSpace: "pre-wrap" }}>{msg.content}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

export default App;
