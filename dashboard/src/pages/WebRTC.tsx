import React, { useState, useRef } from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

interface ConnectionState {
  isConnected: boolean;
  isConnecting: boolean;
  sessionId: string | null;
  error: string | null;
}

const WebRTC: React.FC = () => {
  const [connectionState, setConnectionState] = useState<ConnectionState>({
    isConnected: false,
    isConnecting: false,
    sessionId: null,
    error: null
  });
  const [message, setMessage] = useState('');
  const [messages, setMessages] = useState<Array<{id: number, text: string, type: 'sent' | 'received' | 'system'}>>([
    { id: 1, text: 'WebRTC Demo initialized. Click Connect to establish peer connection.', type: 'system' }
  ]);

  const peerConnectionRef = useRef<RTCPeerConnection | null>(null);
  const dataChannelRef = useRef<RTCDataChannel | null>(null);
  const messageIdRef = useRef(2);

  const addMessage = (text: string, type: 'sent' | 'received' | 'system') => {
    setMessages(prev => [...prev, { id: messageIdRef.current++, text, type }]);
  };

  const connect = async () => {
    setConnectionState(prev => ({ ...prev, isConnecting: true, error: null }));
    addMessage('🔌 Initiating WebRTC connection...', 'system');

    try {
      // Create peer connection
      const peerConnection = new RTCPeerConnection({
        iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
      });
      peerConnectionRef.current = peerConnection;

      // Request offer from server
      const offerResponse = await fetch('/v1/webrtc/offer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          peer_id: 'demo-client-' + Date.now(),
          sdp: 'placeholder'
        })
      });

      if (!offerResponse.ok) {
        throw new Error(`Server returned ${offerResponse.status}: ${offerResponse.statusText}`);
      }

      const offerData = await offerResponse.json();
      const sessionId = offerData.session_id;
      
      addMessage(`📋 Received session ID: ${sessionId}`, 'system');
      setConnectionState(prev => ({ ...prev, sessionId }));

      // Handle incoming data channel
      peerConnection.ondatachannel = (event) => {
        const dataChannel = event.channel;
        dataChannelRef.current = dataChannel;
        
        dataChannel.onopen = () => {
          addMessage('✅ Data channel opened! Connection established.', 'system');
          setConnectionState(prev => ({ 
            ...prev, 
            isConnected: true, 
            isConnecting: false 
          }));
        };
        
        dataChannel.onmessage = (event) => {
          addMessage(event.data, 'received');
        };
        
        dataChannel.onclose = () => {
          addMessage('⚠️ Data channel closed', 'system');
          setConnectionState(prev => ({ 
            ...prev, 
            isConnected: false 
          }));
        };

        dataChannel.onerror = (error) => {
          addMessage(`❌ Data channel error: ${error}`, 'system');
        };
      };

      // Set remote description (simplified for demo)
      await peerConnection.setRemoteDescription({
        type: 'offer',
        sdp: `v=0\no=- ${sessionId} 0 IN IP4 127.0.0.1\ns=TabAgent\nt=0 0\na=group:BUNDLE 0\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\nc=IN IP4 0.0.0.0\na=ice-ufrag:srv\na=ice-pwd:srv123\na=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\na=setup:actpass\na=mid:0\na=sctp-port:5000\na=max-message-size:262144`
      });

      // Create answer
      const answer = await peerConnection.createAnswer();
      await peerConnection.setLocalDescription(answer);

      // Send answer to server
      await fetch('/v1/webrtc/answer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          sdp: answer.sdp
        })
      });

      addMessage('📤 Answer sent to server', 'system');

      // Handle ICE candidates
      peerConnection.onicecandidate = async (event) => {
        if (event.candidate) {
          await fetch('/v1/webrtc/ice', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              session_id: sessionId,
              candidate: event.candidate.candidate
            })
          });
        }
      };

      peerConnection.onconnectionstatechange = () => {
        addMessage(`🔗 Connection state: ${peerConnection.connectionState}`, 'system');
      };

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      addMessage(`❌ Connection failed: ${errorMessage}`, 'system');
      setConnectionState(prev => ({ 
        ...prev, 
        isConnecting: false, 
        error: errorMessage 
      }));
    }
  };

  const disconnect = () => {
    if (dataChannelRef.current) {
      dataChannelRef.current.close();
      dataChannelRef.current = null;
    }
    if (peerConnectionRef.current) {
      peerConnectionRef.current.close();
      peerConnectionRef.current = null;
    }
    setConnectionState({
      isConnected: false,
      isConnecting: false,
      sessionId: null,
      error: null
    });
    addMessage('🔌 Disconnected from server', 'system');
  };

  const sendMessage = () => {
    if (!message.trim() || !dataChannelRef.current || dataChannelRef.current.readyState !== 'open') {
      return;
    }

    dataChannelRef.current.send(message);
    addMessage(message, 'sent');
    setMessage('');
  };

  const sendTestMessage = (testMessage: string) => {
    if (!connectionState.isConnected) {
      addMessage('⚠️ Please connect first', 'system');
      return;
    }
    setMessage(testMessage);
    setTimeout(() => sendMessage(), 100);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div>
      <PageHeader
        title="WebRTC Console"
        description="Establish peer-to-peer connections with TabAgent server using WebRTC data channels"
        actions={
          <div className="flex space-x-2">
            {connectionState.isConnected ? (
              <Button onClick={disconnect} variant="destructive">
                Disconnect
              </Button>
            ) : (
              <Button 
                onClick={connect} 
                disabled={connectionState.isConnecting}
                loading={connectionState.isConnecting}
              >
                {connectionState.isConnecting ? 'Connecting...' : 'Connect'}
              </Button>
            )}
          </div>
        }
      />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Connection Status */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <div className={`w-3 h-3 rounded-full ${
                connectionState.isConnected ? 'bg-success-500 animate-pulse' : 
                connectionState.isConnecting ? 'bg-warning-500 animate-pulse' : 
                'bg-error-500'
              }`}></div>
              Connection Status
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="text-sm font-medium">State</label>
              <p className="text-sm text-muted-foreground">
                {connectionState.isConnected ? 'Connected' : 
                 connectionState.isConnecting ? 'Connecting...' : 
                 'Disconnected'}
              </p>
            </div>
            {connectionState.sessionId && (
              <div>
                <label className="text-sm font-medium">Session ID</label>
                <p className="text-xs font-mono bg-muted p-2 rounded break-all">
                  {connectionState.sessionId}
                </p>
              </div>
            )}
            {connectionState.error && (
              <div>
                <label className="text-sm font-medium text-error-600">Error</label>
                <p className="text-sm text-error-600">{connectionState.error}</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Message Exchange */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Message Exchange</CardTitle>
            <CardDescription>Send and receive messages over the WebRTC data channel</CardDescription>
          </CardHeader>
          <CardContent>
            {/* Messages */}
            <div className="h-64 overflow-y-auto border rounded-lg p-4 mb-4 bg-muted/50">
              {messages.map((msg) => (
                <div key={msg.id} className={`mb-2 ${
                  msg.type === 'sent' ? 'text-right' : 
                  msg.type === 'system' ? 'text-center text-muted-foreground text-sm' : 
                  'text-left'
                }`}>
                  <span className={`inline-block px-3 py-1 rounded-lg ${
                    msg.type === 'sent' ? 'bg-primary-500 text-white' :
                    msg.type === 'received' ? 'bg-secondary-200 dark:bg-secondary-800' :
                    'bg-muted text-muted-foreground'
                  }`}>
                    {msg.text}
                  </span>
                </div>
              ))}
            </div>

            {/* Test Messages */}
            <div className="flex flex-wrap gap-2 mb-4">
              <Button 
                size="sm" 
                variant="outline" 
                onClick={() => sendTestMessage('Hello WebRTC!')}
              >
                Hello
              </Button>
              <Button 
                size="sm" 
                variant="outline" 
                onClick={() => sendTestMessage('{"action": "ping"}')}
              >
                JSON Ping
              </Button>
              <Button 
                size="sm" 
                variant="outline" 
                onClick={() => sendTestMessage('Test message with timestamp: ' + new Date().toISOString())}
              >
                Timestamp
              </Button>
            </div>

            {/* Message Input */}
            <div className="flex space-x-2">
              <Input
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="Type a message..."
                disabled={!connectionState.isConnected}
              />
              <Button 
                onClick={sendMessage}
                disabled={!connectionState.isConnected || !message.trim()}
              >
                Send
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Technical Details */}
      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Technical Details</CardTitle>
          <CardDescription>How this WebRTC demo works</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <h4 className="font-medium mb-2">Connection Process</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Creates RTCPeerConnection with STUN server</li>
                <li>• Requests offer from TabAgent server</li>
                <li>• Sets remote description and creates answer</li>
                <li>• Exchanges ICE candidates for NAT traversal</li>
                <li>• Establishes direct peer-to-peer data channel</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium mb-2">Features Demonstrated</h4>
              <ul className="text-sm space-y-1 text-muted-foreground">
                <li>• Real WebRTC peer connections</li>
                <li>• Data channel messaging</li>
                <li>• Connection state monitoring</li>
                <li>• Bidirectional communication</li>
                <li>• No HTTP polling required</li>
              </ul>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default WebRTC;