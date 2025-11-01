import React, { useState, useRef, useEffect } from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

interface Message {
  id: number;
  content: string;
  type: 'user' | 'assistant' | 'system';
  timestamp: Date;
}

interface ConnectionState {
  isConnected: boolean;
  isConnecting: boolean;
  sessionId: string | null;
}

const Chat: React.FC = () => {
  const [connectionState, setConnectionState] = useState<ConnectionState>({
    isConnected: false,
    isConnecting: false,
    sessionId: null
  });
  const [message, setMessage] = useState('');
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 1,
      content: 'Welcome! Connect to start chatting with TabAgent AI.',
      type: 'system',
      timestamp: new Date()
    }
  ]);
  const [isTyping, setIsTyping] = useState(false);

  const peerConnectionRef = useRef<RTCPeerConnection | null>(null);
  const dataChannelRef = useRef<RTCDataChannel | null>(null);
  const messageIdRef = useRef(2);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const addMessage = (content: string, type: 'user' | 'assistant' | 'system') => {
    const newMessage: Message = {
      id: messageIdRef.current++,
      content,
      type,
      timestamp: new Date()
    };
    setMessages(prev => [...prev, newMessage]);
  };

  const connect = async () => {
    setConnectionState(prev => ({ ...prev, isConnecting: true }));
    addMessage('🔌 Connecting to TabAgent via WebRTC...', 'system');

    try {
      // Create peer connection
      const peerConnection = new RTCPeerConnection({
        iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
      });
      peerConnectionRef.current = peerConnection;

      // Request offer
      const offerResponse = await fetch('/v1/webrtc/offer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          peer_id: 'chat-client-' + Date.now(),
          sdp: 'placeholder'
        })
      });

      if (!offerResponse.ok) throw new Error('Failed to get offer');

      const offerData = await offerResponse.json();
      const sessionId = offerData.session_id;
      setConnectionState(prev => ({ ...prev, sessionId }));

      // Handle data channel
      peerConnection.ondatachannel = (event) => {
        const dataChannel = event.channel;
        dataChannelRef.current = dataChannel;
        
        dataChannel.onopen = () => {
          setConnectionState(prev => ({ 
            ...prev, 
            isConnected: true, 
            isConnecting: false 
          }));
          addMessage('✅ Connected! Start chatting below.', 'system');
        };
        
        dataChannel.onmessage = (event) => {
          setIsTyping(false);
          try {
            const response = JSON.parse(event.data);
            // Handle different response types
            if (response.content) {
              addMessage(response.content, 'assistant');
            } else if (response.message) {
              addMessage(response.message, 'assistant');
            } else {
              addMessage(JSON.stringify(response, null, 2), 'assistant');
            }
          } catch {
            addMessage(event.data, 'assistant');
          }
        };
        
        dataChannel.onclose = () => {
          setConnectionState(prev => ({ 
            ...prev, 
            isConnected: false 
          }));
          addMessage('⚠️ Connection closed', 'system');
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

      // Send answer
      await fetch('/v1/webrtc/answer', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          session_id: sessionId,
          sdp: answer.sdp
        })
      });

      // Handle ICE
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

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      addMessage(`❌ Connection failed: ${errorMessage}`, 'system');
      setConnectionState(prev => ({ 
        ...prev, 
        isConnecting: false 
      }));
    }
  };

  const disconnect = () => {
    if (dataChannelRef.current) dataChannelRef.current.close();
    if (peerConnectionRef.current) peerConnectionRef.current.close();
    dataChannelRef.current = null;
    peerConnectionRef.current = null;
    setConnectionState({
      isConnected: false,
      isConnecting: false,
      sessionId: null
    });
    addMessage('🔌 Disconnected from server', 'system');
  };

  const sendMessage = () => {
    if (!message.trim() || !dataChannelRef.current || dataChannelRef.current.readyState !== 'open') {
      return;
    }

    addMessage(message, 'user');
    setIsTyping(true);

    // Send as chat completion request
    const request = {
      action: 'chat_completion',
      model: 'default',
      messages: [{ role: 'user', content: message }],
      max_tokens: 500,
      temperature: 0.7
    };

    try {
      dataChannelRef.current.send(JSON.stringify(request));
      setMessage('');
    } catch (error) {
      setIsTyping(false);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      addMessage(`❌ Send failed: ${errorMessage}`, 'system');
    }
  };

  const sendQuickPrompt = (prompt: string) => {
    if (!connectionState.isConnected) {
      addMessage('⚠️ Please connect first', 'system');
      return;
    }
    setMessage(prompt);
    setTimeout(() => sendMessage(), 100);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div>
      <PageHeader
        title="AI Chat Interface"
        description="Chat with AI models through TabAgent server using real-time WebRTC connections"
        actions={
          <div className="flex items-center space-x-3">
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${
                connectionState.isConnected ? 'bg-success-500 animate-pulse' : 
                connectionState.isConnecting ? 'bg-warning-500 animate-pulse' : 
                'bg-error-500'
              }`}></div>
              <span className="text-sm text-muted-foreground">
                {connectionState.isConnected ? 'Connected' : 
                 connectionState.isConnecting ? 'Connecting...' : 
                 'Disconnected'}
              </span>
            </div>
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
                Connect via WebRTC
              </Button>
            )}
          </div>
        }
      />

      <Card className="h-[600px] flex flex-col">
        <CardHeader className="flex-shrink-0">
          <CardTitle className="flex items-center gap-2">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            Chat Interface
          </CardTitle>
          <CardDescription>WebRTC-powered live chat with ultra-low latency responses</CardDescription>
        </CardHeader>

        {/* Chat Messages */}
        <CardContent className="flex-1 flex flex-col min-h-0">
          <div className="flex-1 overflow-y-auto space-y-4 mb-4">
            {messages.map((msg) => (
              <div key={msg.id} className={`flex ${
                msg.type === 'user' ? 'justify-end' : 
                msg.type === 'system' ? 'justify-center' : 
                'justify-start'
              }`}>
                <div className={`max-w-[70%] ${msg.type === 'system' ? 'max-w-full' : ''}`}>
                  <div className={`px-4 py-2 rounded-lg ${
                    msg.type === 'user' 
                      ? 'bg-primary-500 text-white rounded-br-sm' 
                      : msg.type === 'assistant'
                      ? 'bg-muted text-foreground rounded-bl-sm'
                      : 'bg-muted/50 text-muted-foreground text-center text-sm'
                  }`}>
                    {msg.content}
                  </div>
                  {msg.type !== 'system' && (
                    <div className={`text-xs text-muted-foreground mt-1 ${
                      msg.type === 'user' ? 'text-right' : 'text-left'
                    }`}>
                      {formatTime(msg.timestamp)}
                    </div>
                  )}
                </div>
              </div>
            ))}
            
            {/* Typing Indicator */}
            {isTyping && (
              <div className="flex justify-start">
                <div className="bg-muted text-foreground px-4 py-2 rounded-lg rounded-bl-sm">
                  <div className="flex space-x-1">
                    <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce"></div>
                    <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce" style={{ animationDelay: '0.1s' }}></div>
                    <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce" style={{ animationDelay: '0.2s' }}></div>
                  </div>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* Quick Prompts */}
          <div className="flex flex-wrap gap-2 mb-4">
            <Button 
              size="sm" 
              variant="outline" 
              onClick={() => sendQuickPrompt('Tell me about yourself')}
              disabled={!connectionState.isConnected}
            >
              Tell me about yourself
            </Button>
            <Button 
              size="sm" 
              variant="outline" 
              onClick={() => sendQuickPrompt('What can you do?')}
              disabled={!connectionState.isConnected}
            >
              What can you do?
            </Button>
            <Button 
              size="sm" 
              variant="outline" 
              onClick={() => sendQuickPrompt('Explain quantum computing')}
              disabled={!connectionState.isConnected}
            >
              Explain quantum computing
            </Button>
            <Button 
              size="sm" 
              variant="outline" 
              onClick={() => sendQuickPrompt('Write a haiku about AI')}
              disabled={!connectionState.isConnected}
            >
              Write a haiku
            </Button>
          </div>

          {/* Message Input */}
          <div className="flex space-x-2">
            <Input
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Type your message..."
              disabled={!connectionState.isConnected}
              className="flex-1"
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

      {/* Technical Info */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        <Card>
          <CardHeader>
            <CardTitle>Features</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                WebRTC-powered live chat
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                No HTTP polling required
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                Ultra-low latency responses
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                Real-time typing indicators
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                Beautiful chat interface
              </li>
            </ul>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>How It Works</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li>• Establishes WebRTC peer connection</li>
              <li>• Uses data channels for messaging</li>
              <li>• Sends chat completion requests as JSON</li>
              <li>• Receives AI responses in real-time</li>
              <li>• No server polling or WebSocket needed</li>
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Chat;