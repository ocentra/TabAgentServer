import React, { useState } from 'react';
import { Card } from '../../ui/Card';
import { Button } from '../../ui/Button';
import { LoadingSpinner } from '../../ui/LoadingSpinner';
import { useSystemStats, usePerformanceStats, useHealth } from '../../../hooks/useApi';

interface SystemDiagnosticsProps {
  className?: string;
}

interface DiagnosticTest {
  id: string;
  name: string;
  description: string;
  status: 'pass' | 'fail' | 'warning' | 'running' | 'not_run';
  result?: string;
  details?: string;
}

export const SystemDiagnostics: React.FC<SystemDiagnosticsProps> = ({ className = '' }) => {
  const { data: systemStats } = useSystemStats();
  const { data: performanceStats } = usePerformanceStats();
  const { data: health } = useHealth();
  const [runningTests, setRunningTests] = useState<Set<string>>(new Set());
  const [, setTestResults] = useState<Map<string, DiagnosticTest>>(new Map());

  const diagnosticTests: DiagnosticTest[] = [
    {
      id: 'api_connectivity',
      name: 'API Connectivity',
      description: 'Test if the API endpoints are responding correctly',
      status: health ? 'pass' : 'fail',
      result: health ? 'All API endpoints are responding' : 'API endpoints not responding',
    },
    {
      id: 'memory_usage',
      name: 'Memory Usage Check',
      description: 'Verify system memory usage is within acceptable limits',
      status: (systemStats?.memory_usage || 0) < 85 ? 'pass' : (systemStats?.memory_usage || 0) < 95 ? 'warning' : 'fail',
      result: `Memory usage: ${(systemStats?.memory_usage || 0).toFixed(1)}%`,
      details: (systemStats?.memory_usage || 0) > 85 ? 'High memory usage detected. Consider restarting services or adding more RAM.' : undefined,
    },
    {
      id: 'cpu_usage',
      name: 'CPU Usage Check',
      description: 'Monitor CPU usage for performance issues',
      status: (systemStats?.cpu_usage || 0) < 80 ? 'pass' : (systemStats?.cpu_usage || 0) < 95 ? 'warning' : 'fail',
      result: `CPU usage: ${(systemStats?.cpu_usage || 0).toFixed(1)}%`,
      details: (systemStats?.cpu_usage || 0) > 80 ? 'High CPU usage detected. Check for resource-intensive processes.' : undefined,
    },
    {
      id: 'disk_space',
      name: 'Disk Space Check',
      description: 'Ensure adequate disk space is available',
      status: (systemStats?.disk_usage || 0) < 85 ? 'pass' : (systemStats?.disk_usage || 0) < 95 ? 'warning' : 'fail',
      result: `Disk usage: ${(systemStats?.disk_usage || 0).toFixed(1)}%`,
      details: (systemStats?.disk_usage || 0) > 85 ? 'Low disk space detected. Consider cleaning up old files or expanding storage.' : undefined,
    },
    {
      id: 'response_time',
      name: 'Response Time Check',
      description: 'Verify API response times are acceptable',
      status: (performanceStats?.average_response_time || 0) < 500 ? 'pass' : (performanceStats?.average_response_time || 0) < 1000 ? 'warning' : 'fail',
      result: `Average response time: ${(performanceStats?.average_response_time || 0).toFixed(0)}ms`,
      details: (performanceStats?.average_response_time || 0) > 500 ? 'Slow response times detected. Check server load and network connectivity.' : undefined,
    },
    {
      id: 'error_rate',
      name: 'Error Rate Check',
      description: 'Monitor system error rates',
      status: (performanceStats?.error_rate || 0) < 1 ? 'pass' : (performanceStats?.error_rate || 0) < 5 ? 'warning' : 'fail',
      result: `Error rate: ${(performanceStats?.error_rate || 0).toFixed(2)}%`,
      details: (performanceStats?.error_rate || 0) > 1 ? 'Elevated error rate detected. Check logs for specific error patterns.' : undefined,
    },
    {
      id: 'service_health',
      name: 'Service Health Check',
      description: 'Verify all required services are running',
      status: health?.services?.http && health?.services?.webrtc ? 'pass' : 'fail',
      result: `HTTP: ${health?.services?.http ? 'Running' : 'Stopped'}, WebRTC: ${health?.services?.webrtc ? 'Running' : 'Stopped'}`,
      details: !health?.services?.http || !health?.services?.webrtc ? 'One or more critical services are not running.' : undefined,
    }
  ];

  const runDiagnosticTest = async (testId: string) => {
    setRunningTests(prev => new Set([...prev, testId]));
    
    // Simulate running a test
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    const test = diagnosticTests.find(t => t.id === testId);
    if (test) {
      setTestResults(prev => new Map([...prev, [testId, { ...test, status: 'pass' }]]));
    }
    
    setRunningTests(prev => {
      const newSet = new Set(prev);
      newSet.delete(testId);
      return newSet;
    });
  };

  const runAllTests = async () => {
    for (const test of diagnosticTests) {
      if (!runningTests.has(test.id)) {
        await runDiagnosticTest(test.id);
      }
    }
  };

  const getStatusIcon = (status: DiagnosticTest['status']) => {
    switch (status) {
      case 'pass':
        return (
          <svg className="w-5 h-5 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
        );
      case 'warning':
        return (
          <svg className="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
        );
      case 'fail':
        return (
          <svg className="w-5 h-5 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        );
      case 'running':
        return <LoadingSpinner size="sm" />;
      default:
        return (
          <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
    }
  };

  const getStatusColor = (status: DiagnosticTest['status']) => {
    switch (status) {
      case 'pass':
        return 'text-green-700 bg-green-50 border-green-200 dark:text-green-400 dark:bg-green-900/20 dark:border-green-800';
      case 'warning':
        return 'text-yellow-700 bg-yellow-50 border-yellow-200 dark:text-yellow-400 dark:bg-yellow-900/20 dark:border-yellow-800';
      case 'fail':
        return 'text-red-700 bg-red-50 border-red-200 dark:text-red-400 dark:bg-red-900/20 dark:border-red-800';
      case 'running':
        return 'text-blue-700 bg-blue-50 border-blue-200 dark:text-blue-400 dark:bg-blue-900/20 dark:border-blue-800';
      default:
        return 'text-gray-700 bg-gray-50 border-gray-200 dark:text-gray-400 dark:bg-gray-800 dark:border-gray-700';
    }
  };

  const passedTests = diagnosticTests.filter(test => test.status === 'pass').length;
  const totalTests = diagnosticTests.length;

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            System Diagnostics
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
            {passedTests}/{totalTests} tests passing
          </p>
        </div>
        <Button onClick={runAllTests} variant="default" size="sm">
          Run All Tests
        </Button>
      </div>

      {/* Overall Status */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Overall System Health
          </h3>
          <div className={`px-3 py-1 rounded-full text-sm font-medium ${
            passedTests === totalTests 
              ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
              : passedTests > totalTests * 0.7
              ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
              : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
          }`}>
            {passedTests === totalTests 
              ? 'Healthy' 
              : passedTests > totalTests * 0.7 
              ? 'Warning' 
              : 'Critical'}
          </div>
        </div>
        
        <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
          <div 
            className={`h-2 rounded-full transition-all duration-300 ${
              passedTests === totalTests 
                ? 'bg-green-500' 
                : passedTests > totalTests * 0.7 
                ? 'bg-yellow-500' 
                : 'bg-red-500'
            }`}
            style={{ width: `${(passedTests / totalTests) * 100}%` }}
          />
        </div>
        
        <div className="flex justify-between text-sm text-gray-600 dark:text-gray-400 mt-2">
          <span>{passedTests} tests passed</span>
          <span>{totalTests - passedTests} issues detected</span>
        </div>
      </Card>

      {/* Diagnostic Tests */}
      <Card className="p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Diagnostic Tests
        </h3>
        
        <div className="space-y-4">
          {diagnosticTests.map((test) => {
            const isRunning = runningTests.has(test.id);
            const currentStatus = isRunning ? 'running' : test.status;
            
            return (
              <div
                key={test.id}
                className={`border rounded-lg p-4 ${getStatusColor(currentStatus)}`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start space-x-3 flex-1">
                    <div className="flex-shrink-0 mt-0.5">
                      {getStatusIcon(currentStatus)}
                    </div>
                    <div className="flex-1">
                      <h4 className="font-medium">{test.name}</h4>
                      <p className="text-sm opacity-75 mt-1">{test.description}</p>
                      {test.result && (
                        <p className="text-sm font-medium mt-2">{test.result}</p>
                      )}
                      {test.details && (
                        <p className="text-sm mt-2 p-2 bg-black/10 dark:bg-white/10 rounded">
                          {test.details}
                        </p>
                      )}
                    </div>
                  </div>
                  <Button
                    onClick={() => runDiagnosticTest(test.id)}
                    variant="secondary"
                    size="sm"
                    disabled={isRunning}
                  >
                    {isRunning ? 'Running...' : 'Retest'}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </Card>

      {/* Quick Actions */}
      <Card className="p-6">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          Quick Actions
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <Button variant="secondary" className="justify-start">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            Export Diagnostics
          </Button>
          
          <Button variant="secondary" className="justify-start">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Restart Services
          </Button>
          
          <Button variant="secondary" className="justify-start">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            View Logs
          </Button>
          
          <Button variant="secondary" className="justify-start">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Settings
          </Button>
        </div>
      </Card>
    </div>
  );
};