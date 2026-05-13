import { useState, useEffect, useRef } from "react";
import "./App.css";

function App() {
  const [clientId, setClientId] = useState("");
  const [targetId, setTargetId] = useState("");
  const [status, setStatus] = useState("Disconnected");
  const [messages, setMessages] = useState<string[]>([]);
  
  const wsRef = useRef<WebSocket | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const dcRef = useRef<RTCDataChannel | null>(null);

  useEffect(() => {
    const id = Math.floor(100000000 + Math.random() * 900000000).toString();
    setClientId(id);
    
    // Connect to Signaling Server
    const ws = new WebSocket("ws://localhost:8080/ws");
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus("Connected to Signaling");
      ws.send(JSON.stringify({ type: "Register", id }));
    };

    ws.onmessage = async (event) => {
      const msg = JSON.parse(event.data);
      console.log("Signaling message received:", msg);

      if (msg.type === "Offer") {
        setStatus(`Incoming connection from ${msg.from}...`);
        await handleOffer(msg.from, msg.sdp);
      } else if (msg.type === "Answer") {
        setStatus("Connection established!");
        await handleAnswer(msg.sdp);
      } else if (msg.type === "IceCandidate") {
        await handleIceCandidate(msg.candidate);
      }
    };

    ws.onclose = () => setStatus("Disconnected from Signaling");

    return () => {
      ws.close();
      pcRef.current?.close();
    };
  }, []);

  const createPeerConnection = (target: string) => {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: "stun:stun.l.google.com:19302" }]
    });
    
    pc.onicecandidate = (event) => {
      if (event.candidate && wsRef.current) {
        wsRef.current.send(JSON.stringify({
          type: "IceCandidate",
          target,
          candidate: JSON.stringify(event.candidate)
        }));
      }
    };

    pc.ondatachannel = (event) => {
      setupDataChannel(event.channel);
    };

    pcRef.current = pc;
    return pc;
  };

  const setupDataChannel = (dc: RTCDataChannel) => {
    dc.onopen = () => setStatus("P2P Data Channel Open!");
    dc.onmessage = (event) => {
      setMessages((prev) => [...prev, `Remote: ${event.data}`]);
    };
    dcRef.current = dc;
  };

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!targetId || !wsRef.current) return;
    setStatus(`Connecting to ${targetId}...`);
    
    const pc = createPeerConnection(targetId);
    const dc = pc.createDataChannel("enidesk-control");
    setupDataChannel(dc);

    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);

    wsRef.current.send(JSON.stringify({
      type: "Offer",
      target: targetId,
      sdp: JSON.stringify(offer)
    }));
  };

  const handleOffer = async (from: string, sdpStr: string) => {
    const pc = createPeerConnection(from);
    const offer = JSON.parse(sdpStr);
    await pc.setRemoteDescription(new RTCSessionDescription(offer));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    wsRef.current?.send(JSON.stringify({
      type: "Answer",
      target: from,
      sdp: JSON.stringify(answer)
    }));
  };

  const handleAnswer = async (sdpStr: string) => {
    const answer = JSON.parse(sdpStr);
    await pcRef.current?.setRemoteDescription(new RTCSessionDescription(answer));
  };

  const handleIceCandidate = async (candidateStr: string) => {
    const candidate = JSON.parse(candidateStr);
    await pcRef.current?.addIceCandidate(new RTCIceCandidate(candidate));
  };

  const sendTestMessage = () => {
    if (dcRef.current && dcRef.current.readyState === "open") {
      dcRef.current.send("Hello from peer!");
      setMessages((prev) => [...prev, "You: Hello from peer!"]);
    }
  };

  return (
    <main className="container" style={{ padding: "2rem", fontFamily: "sans-serif" }}>
      <h1>EniDesk</h1>
      
      <div style={{ marginBottom: "2rem", padding: "1rem", backgroundColor: "#f0f0f0", borderRadius: "8px" }}>
        <h2>Your Address</h2>
        <div style={{ fontSize: "2rem", fontWeight: "bold", letterSpacing: "2px" }}>
          {clientId || "Loading..."}
        </div>
        <p>Status: {status}</p>
      </div>

      <div style={{ padding: "1rem", border: "1px solid #ccc", borderRadius: "8px", marginBottom: "2rem" }}>
        <h2>Remote Desk</h2>
        <form onSubmit={handleConnect} style={{ display: "flex", gap: "1rem", marginTop: "1rem" }}>
          <input
            type="text"
            value={targetId}
            onChange={(e) => setTargetId(e.currentTarget.value)}
            placeholder="Remote Address"
            style={{ padding: "0.5rem", fontSize: "1rem", flex: 1 }}
          />
          <button type="submit" style={{ padding: "0.5rem 1rem", fontSize: "1rem", cursor: "pointer" }}>
            Connect
          </button>
        </form>
      </div>

      {status === "P2P Data Channel Open!" && (
        <div style={{ padding: "1rem", border: "1px solid #4CAF50", borderRadius: "8px" }}>
          <h2>P2P Chat (Test)</h2>
          <button onClick={sendTestMessage} style={{ padding: "0.5rem", marginBottom: "1rem" }}>Send Test Message</button>
          <div style={{ maxHeight: "150px", overflowY: "auto", background: "#f9f9f9", padding: "0.5rem" }}>
            {messages.map((m, i) => <div key={i}>{m}</div>)}
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
