import React, { useState, useEffect } from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';

interface ServerStatus {
  httpOnline: boolean;
  webrtcOnline: boolean;
  detectedPort: string;
  serverMode: string;
}

const API: React.FC = () => {
  const [serverStatus, setServerStatus] = useState<ServerStatus>({
    httpOnline: false,
    webrtcOnline: false,
    detectedPort: '3000',
    serverMode: 'Detecting...'
  });
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkServerStatus();
  }, []);

  const checkServerStatus = async () => {
    setLoading(true);
    const currentPort = window.location.port || '80';
    
    try {
      // Check HTTP API
      let httpOnline = false;
      try {
        const response = await fetch('/v1/health');
        httpOnline = response.ok;
      } catch {
        httpOnline = false;
      }

      // Check WebRTC endpoint
      let webrtcOnline = false;
      try {
        const response = await fetch('/v1/webrtc/session/test');
        webrtcOnline = response.status === 404 || response.status === 200;
      } catch {
        webrtcOnline = false;
      }

      // Determine mode
      let mode = 'Unknown';
      if (httpOnline && webrtcOnline) {
        mode = 'ALL';
      } else if (httpOnline) {
        mode = currentPort === '9000' ? 'WebRTC' : 'HTTP';
      } else if (webrtcOnline) {
        mode = 'WebRTC';
      }

      setServerStatus({
        httpOnline,
        webrtcOnline,
        detectedPort: currentPort,
        serverMode: mode
      });
    } catch (error) {
      console.error('Failed to check server status:', error);
    } finally {
      setLoading(false);
    }
  };

  const getStatusBadge = (online: boolean) => (
    <span className={`px-2 py-1 text-xs rounded-full ${
      online 
        ? 'bg-success-100 text-success-800 dark:bg-success-900 dark:text-success-200' 
        : 'bg-error-100 text-error-800 dark:bg-error-900 dark:text-error-200'
    }`}>
      {online ? '✓ Online' : '✗ Offline'}
    </span>
  );

  return (
    <div>
      <PageHeader
        title="API Explorer"
        description="Explore and test TabAgent server endpoints, monitor server status, and access documentation"
        actions={
          <Button onClick={checkServerStatus} disabled={loading}>
            {loading ? 'Checking...' : 'Refresh Status'}
          </Button>
        }
      />

      {/* Server Status */}
      <Card className="mb-6">
        <CardHeader>
          <CardTitle>Server Status</CardTitle>
          <CardDescription>Current server configuration and endpoint availability</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="text-center p-4 bg-muted rounded-lg">
              <h3 className="text-sm font-medium text-muted-foreground mb-2">SERVER MODE</h3>
              <div className="text-lg font-bold">
                <span className="px-3 py-1 bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200 rounded-full text-sm">
                  {serverStatus.serverMode}
                </span>
              </div>
            </div>
            <div className="text-center p-4 bg-muted rounded-lg">
              <h3 className="text-sm font-medium text-muted-foreground mb-2">HTTP API</h3>
              <div className="text-lg font-bold">
                {getStatusBadge(serverStatus.httpOnline)}
              </div>
            </div>
            <div className="text-center p-4 bg-muted rounded-lg">
              <h3 className="text-sm font-medium text-muted-foreground mb-2">WEBRTC</h3>
              <div className="text-lg font-bold">
                {getStatusBadge(serverStatus.webrtcOnline)}
              </div>
            </div>
            <div className="text-center p-4 bg-muted rounded-lg">
              <h3 className="text-sm font-medium text-muted-foreground mb-2">DETECTED PORT</h3>
              <div className="text-lg font-bold">{serverStatus.detectedPort}</div>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* API Documentation */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
              </svg>
              API Documentation
            </CardTitle>
            <CardDescription>
              Access interactive API documentation (requires TabAgent server with documentation endpoints enabled)
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={async () => {
                try {
                  const response = await fetch('/swagger-ui/');
                  if (response.ok) {
                    window.open('/swagger-ui/', '_blank');
                  } else {
                    alert('Swagger UI not available. Make sure TabAgent server is running with API documentation enabled.');
                  }
                } catch {
                  alert('Cannot connect to server. Please check if TabAgent server is running.');
                }
              }}
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
              Swagger UI
              {!serverStatus.httpOnline && <span className="ml-2 text-xs text-error-500">(Offline)</span>}
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={async () => {
                try {
                  const response = await fetch('/rapidoc/');
                  if (response.ok) {
                    window.open('/rapidoc/', '_blank');
                  } else {
                    alert('RapiDoc not available. Make sure TabAgent server is running with API documentation enabled.');
                  }
                } catch {
                  alert('Cannot connect to server. Please check if TabAgent server is running.');
                }
              }}
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              RapiDoc
              {!serverStatus.httpOnline && <span className="ml-2 text-xs text-error-500">(Offline)</span>}
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={async () => {
                try {
                  const response = await fetch('/redoc/');
                  if (response.ok) {
                    window.open('/redoc/', '_blank');
                  } else {
                    alert('ReDoc not available. Make sure TabAgent server is running with API documentation enabled.');
                  }
                } catch {
                  alert('Cannot connect to server. Please check if TabAgent server is running.');
                }
              }}
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              ReDoc
              {!serverStatus.httpOnline && <span className="ml-2 text-xs text-error-500">(Offline)</span>}
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={async () => {
                try {
                  const response = await fetch('/api-doc/openapi.json');
                  if (response.ok) {
                    window.open('/api-doc/openapi.json', '_blank');
                  } else {
                    alert('OpenAPI spec not available. Make sure TabAgent server is running with API documentation enabled.');
                  }
                } catch {
                  alert('Cannot connect to server. Please check if TabAgent server is running.');
                }
              }}
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
              </svg>
              OpenAPI Spec (JSON)
              {!serverStatus.httpOnline && <span className="ml-2 text-xs text-error-500">(Offline)</span>}
            </Button>
          </CardContent>
        </Card>

        {/* Quick API Tests */}
        <Card>
          <CardTitle className="flex items-center gap-2 p-6 pb-3">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            Quick API Tests
          </CardTitle>
          <CardDescription className="px-6 pb-3">Test common endpoints directly from your browser</CardDescription>
          <CardContent className="space-y-3">
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => window.open('/v1/health', '_blank')}
            >
              <svg className="w-4 h-4 mr-2 text-success-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Health Check
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => window.open('/v1/system', '_blank')}
            >
              <svg className="w-4 h-4 mr-2 text-primary-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
              </svg>
              System Info
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => window.open('/v1/models', '_blank')}
            >
              <svg className="w-4 h-4 mr-2 text-secondary-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364-.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
              List Models
            </Button>
            <Button 
              className="w-full justify-start" 
              variant="outline"
              onClick={() => window.open('/v1/stats', '_blank')}
            >
              <svg className="w-4 h-4 mr-2 text-warning-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              Server Stats
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* Communication Protocols */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mt-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
              HTTP/REST API
            </CardTitle>
            <CardDescription>Port: {serverStatus.detectedPort} (default: 3000)</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                Traditional REST endpoints
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                JSON request/response
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                Perfect for web apps
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-success-500 rounded-full"></span>
                OpenAPI 3.0 compliant
              </li>
            </ul>
            <div className="mt-4 p-2 bg-muted rounded text-xs font-mono">
              --mode http --port 3000
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              WebRTC P2P
            </CardTitle>
            <CardDescription>Port: 9000 (signaling)</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-primary-500 rounded-full"></span>
                Peer-to-peer data channels
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-primary-500 rounded-full"></span>
                Low latency communication
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-primary-500 rounded-full"></span>
                Browser-native support
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-primary-500 rounded-full"></span>
                No extension required
              </li>
            </ul>
            <div className="mt-4 p-2 bg-muted rounded text-xs font-mono">
              --mode webrtc --webrtc-port 9000
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              Native Messaging
            </CardTitle>
            <CardDescription>Protocol: stdin/stdout</CardDescription>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm">
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-secondary-500 rounded-full"></span>
                Chrome extension communication
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-secondary-500 rounded-full"></span>
                Binary-safe messaging
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-secondary-500 rounded-full"></span>
                Length-prefixed JSON
              </li>
              <li className="flex items-center gap-2">
                <span className="w-2 h-2 bg-secondary-500 rounded-full"></span>
                Direct process communication
              </li>
            </ul>
            <div className="mt-4 p-2 bg-muted rounded text-xs font-mono">
              --mode native
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default API;