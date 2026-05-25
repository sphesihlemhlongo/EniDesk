import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [clientId, setClientId] = useState("");
  const [targetId, setTargetId] = useState("");
  const [status, setStatus] = useState("Disconnected");
  const [remoteStream, setRemoteStream] = useState<string | null>(null);
  const [isHost, setIsHost] = useState(false);
  const [myPassword, setMyPassword] = useState("");
  const [targetPassword, setTargetPassword] = useState("");
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  
  // New States
  const [onlineClients, setOnlineClients] = useState<string[]>([]);
  const [incomingConnection, setIncomingConnection] = useState<{from: string, sdp: string} | null>(null);
  
  const wsRef = useRef<WebSocket | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const dcRef = useRef<RTCDataChannel | null>(null);

  useEffect(() => {
    const id = Math.floor(100000000 + Math.random() * 900000000).toString();
    setClientId(id);
    
    const ws = new WebSocket("ws://localhost:8080/ws");
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus("Connected to Signaling");
      ws.send(JSON.stringify({ type: "Register", id }));
    };

    ws.onmessage = async (event) => {
      const msg = JSON.parse(event.data);
      if (msg.type === "ClientList") {
        setOnlineClients(msg.clients.filter((c: string) => c !== id));
      } else if (msg.type === "Offer") {
        // Prompt the host instead of auto-answering
        setIncomingConnection({ from: msg.from, sdp: msg.sdp });
      } else if (msg.type === "Answer") {
        await handleAnswer(msg.sdp);
      } else if (msg.type === "IceCandidate") {
        await handleIceCandidate(msg.candidate);
      } else if (msg.type === "Rejected") {
        setStatus("Connection Rejected by Host");
        pcRef.current?.close();
      } else if (msg.type === "Error") {
        setStatus(msg.message);
      }
    };

    const unlisten = listen("screen-frame", (event) => {
      const frameData = event.payload as string;
      if (dcRef.current && dcRef.current.readyState === "open") {
        dcRef.current.send(JSON.stringify({ type: "frame", data: frameData }));
      }
    });

    return () => {
      ws.close();
      pcRef.current?.close();
      unlisten.then(f => f());
    };
  }, []);

  const updateMyPassword = (pass: string) => {
    setMyPassword(pass);
    invoke("set_password", { password: pass });
  };

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
    dc.onopen = () => {
        setStatus("P2P Connected");
        if (!isHost) {
            setIsAuthenticating(true);
        }
    };
    dc.onmessage = async (event) => {
      const msg = JSON.parse(event.data);
      if (msg.type === "frame") {
        setRemoteStream(`data:image/jpeg;base64,${msg.data}`);
      } else if (msg.type === "input") {
        invoke("inject_input", {
            eventType: msg.event,
            keyOrButton: msg.key,
            x: msg.x,
            y: msg.y
        });
      } else if (msg.type === "auth-request") {
          const success = await invoke("verify_password", { password: msg.password });
          dc.send(JSON.stringify({ type: "auth-response", success }));
      } else if (msg.type === "auth-response") {
          if (msg.success) {
              setIsAuthenticating(false);
              setStatus("Authenticated");
          } else {
              setStatus("Authentication Failed");
              dc.close();
          }
      }
    };
    dcRef.current = dc;
  };

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!targetId || !wsRef.current) return;
    setIsHost(false);
    setStatus(`Connecting to ${targetId} (Waiting for Host)...`);
    
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

  const submitPassword = () => {
      if (dcRef.current && dcRef.current.readyState === "open") {
          dcRef.current.send(JSON.stringify({ type: "auth-request", password: targetPassword }));
      }
  };

  const handleAcceptConnection = async () => {
    if (!incomingConnection) return;
    setIsHost(true);
    setStatus("Accepting connection...");
    
    const pc = createPeerConnection(incomingConnection.from);
    const offer = JSON.parse(incomingConnection.sdp);
    await pc.setRemoteDescription(new RTCSessionDescription(offer));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    wsRef.current?.send(JSON.stringify({
      type: "Answer",
      target: incomingConnection.from,
      sdp: JSON.stringify(answer)
    }));
    
    setIncomingConnection(null);
  };

  const handleRejectConnection = () => {
    if (!incomingConnection) return;
    wsRef.current?.send(JSON.stringify({
      type: "Reject",
      target: incomingConnection.from
    }));
    setIncomingConnection(null);
  };

  const handleAnswer = async (sdpStr: string) => {
    const answer = JSON.parse(sdpStr);
    await pcRef.current?.setRemoteDescription(new RTCSessionDescription(answer));
  };

  const handleIceCandidate = async (candidateStr: string) => {
    const candidate = JSON.parse(candidateStr);
    await pcRef.current?.addIceCandidate(new RTCIceCandidate(candidate));
  };

  const startSharing = async () => {
    await invoke("start_stream", { displayIndex: 0 });
    setStatus("Sharing Screen...");
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isHost && dcRef.current?.readyState === "open" && status === "Authenticated") {
        const rect = e.currentTarget.getBoundingClientRect();
        const x = (e.clientX - rect.left) / rect.width;
        const y = (e.clientY - rect.top) / rect.height;
        dcRef.current.send(JSON.stringify({ type: "input", event: "mousemove", x, y }));
    }
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    if (!isHost && dcRef.current?.readyState === "open" && status === "Authenticated") {
        const button = e.button === 0 ? "left" : e.button === 2 ? "right" : "middle";
        dcRef.current.send(JSON.stringify({ type: "input", event: "mousedown", key: button }));
    }
  };

  return (
    <main className="container">
      <h1>EniDesk</h1>
      
      {!remoteStream && !isAuthenticating && (
        <>
            <div className="section">
                <h2>Your Address: {clientId}</h2>
                <p>Status: {status}</p>
                <input 
                    type="password" 
                    placeholder="Set Access Password (Optional)" 
                    value={myPassword} 
                    onChange={(e) => updateMyPassword(e.target.value)}
                />
                {isHost && status === "P2P Connected" && (
                    <button onClick={startSharing} style={{marginTop: "10px"}}>Start Sharing Screen</button>
                )}
            </div>

            {incomingConnection && (
                <div className="section" style={{ border: "2px solid #d32f2f", backgroundColor: "#fff8f8" }}>
                    <h2>Incoming Connection!</h2>
                    <p>Device <strong>{incomingConnection.from}</strong> wants to connect.</p>
                    <div style={{ display: "flex", gap: "10px", marginTop: "10px" }}>
                        <button onClick={handleAcceptConnection} style={{ backgroundColor: "#4CAF50" }}>Allow</button>
                        <button onClick={handleRejectConnection} style={{ backgroundColor: "#f44336" }}>Deny</button>
                    </div>
                </div>
            )}

            <div className="section">
                <h2>Discover & Connect</h2>
                
                {onlineClients.length > 0 ? (
                    <div style={{ marginBottom: "15px", textAlign: "left" }}>
                        <p style={{ margin: "0 0 5px 0", fontWeight: "bold" }}>Online Devices on Network:</p>
                        <ul style={{ listStyleType: "none", padding: 0, margin: 0, maxHeight: "150px", overflowY: "auto", border: "1px solid #ccc", borderRadius: "4px" }}>
                            {onlineClients.map(id => (
                                <li 
                                    key={id} 
                                    onClick={() => setTargetId(id)}
                                    style={{ padding: "8px", borderBottom: "1px solid #eee", cursor: "pointer", backgroundColor: targetId === id ? "#e3f2fd" : "white" }}
                                >
                                    🖥️ {id}
                                </li>
                            ))}
                        </ul>
                    </div>
                ) : (
                    <p style={{ fontStyle: "italic", color: "#888" }}>No other devices found online.</p>
                )}

                <form onSubmit={handleConnect}>
                    <input
                        type="text"
                        value={targetId}
                        onChange={(e) => setTargetId(e.currentTarget.value)}
                        placeholder="Select or enter Address"
                    />
                    <button type="submit">Connect</button>
                </form>
            </div>
        </>
      )}

      {isAuthenticating && (
          <div className="section">
              <h2>Authentication Required</h2>
              <p>Enter the password for {targetId}</p>
              <input 
                type="password" 
                value={targetPassword} 
                onChange={(e) => setTargetPassword(e.target.value)} 
                placeholder="Remote Password"
              />
              <button onClick={submitPassword}>Authenticate</button>
          </div>
      )}

      {remoteStream && (
        <div className="viewer">
          <img 
            src={remoteStream} 
            alt="Remote Screen" 
            onMouseMove={handleMouseMove}
            onMouseDown={handleMouseDown}
            style={{ width: "100%", cursor: "crosshair" }}
          />
          <button onClick={() => setRemoteStream(null)}>Disconnect</button>
        </div>
      )}
    </main>
  );
}

export default App;
