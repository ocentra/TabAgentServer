#!/usr/bin/env node

/**
 * TabAgent Development Starter
 * 
 * Orchestrates all services with smart port management
 * Reads service configuration from services.config.ts
 */

import { spawn, type ChildProcess } from 'child_process';
import process from 'process';
import path from 'path';
import { allocatePorts } from './port-manager.js';
import { SERVICES, getStartupOrder } from './services.config.js';
import { detectLibclang } from './detect-libclang.js';

interface RunningService {
  name: string;
  child: ChildProcess;
}

const services: RunningService[] = [];

/**
 * Start a service with proper environment and logging
 */
function startService(
  name: string,
  command: string,
  args: string[],
  cwd: string,
  env: Record<string, string> = {}
): ChildProcess {
  console.log(`\n🚀 Starting ${name}...`);
  
  const child = spawn(command, args, {
    cwd,
    env: { ...process.env, ...env },
    stdio: 'inherit',
    shell: true
  });
  
  child.on('error', (err) => {
    console.error(`❌ ${name} error:`, err);
  });
  
  child.on('exit', (code) => {
    if (code !== 0 && code !== null) {
      console.error(`❌ ${name} exited with code ${code}`);
      cleanup();
      process.exit(code);
    }
  });
  
  services.push({ name, child });
  return child;
}

/**
 * Cleanup all services
 */
function cleanup(): void {
  console.log('\n🛑 Shutting down services...');
  
  for (const { name, child } of services) {
    console.log(`   Stopping ${name}...`);
    try {
      if (process.platform === 'win32') {
        spawn('taskkill', ['/pid', child.pid!.toString(), '/f', '/t']);
      } else {
        child.kill('SIGTERM');
      }
    } catch (err) {
      console.error(`   Failed to stop ${name}:`, (err as Error).message);
    }
  }
  
  console.log('✅ All services stopped\n');
}

/**
 * Main startup sequence (auto-discovers services from config!)
 */
async function main(): Promise<void> {
  console.log('\n╔════════════════════════════════════════╗');
  console.log('║   TabAgent Development Environment    ║');
  console.log('╚════════════════════════════════════════╝');
  
  // Step 0: Auto-detect libclang for Windows Rust builds
  const libclangPath = detectLibclang();
  if (libclangPath) {
    process.env.LIBCLANG_PATH = libclangPath;
    console.log(`   🔧 Set LIBCLANG_PATH for Rust: ${libclangPath}\n`);
  }
  
  // Step 1: Allocate ports
  let ports: Record<string, number>;
  try {
    ports = await allocatePorts();
  } catch (err) {
    console.error('❌ Port allocation failed:', (err as Error).message);
    process.exit(1);
  }
  
  // Step 2: Start services in order (backends first, then frontends)
  const startupOrder = getStartupOrder();
  
  for (const serviceKey of startupOrder) {
    const service = SERVICES[serviceKey];
    const servicePort = ports[serviceKey];
    
    if (!service.dev) {
      console.log(`⏭️  Skipping ${service.name} (no dev config)`);
      continue;
    }
    
    // Build environment variables
    const env: Record<string, string> = {};
    if (service.dev.env) {
      for (const [key, value] of Object.entries(service.dev.env)) {
        let resolved = value;
        
        // Replace port placeholders
        resolved = resolved.replace('{port}', servicePort.toString());
        for (const [otherKey, otherPort] of Object.entries(ports)) {
          resolved = resolved.replace(`{${otherKey}Port}`, otherPort.toString());
        }
        
        env[key] = resolved;
      }
    }
    
    // Explicitly pass LIBCLANG_PATH to Rust builds (Windows bindgen fix)
    if (serviceKey === 'rust' && process.env.LIBCLANG_PATH) {
      env.LIBCLANG_PATH = process.env.LIBCLANG_PATH;
    }
    
    // Build command args (replace port placeholders)
    const args = service.dev.args.map(arg => {
      let resolved = arg;
      resolved = resolved.replace('{port}', servicePort.toString());
      for (const [otherKey, otherPort] of Object.entries(ports)) {
        resolved = resolved.replace(`{${otherKey}Port}`, otherPort.toString());
      }
      return resolved;
    });
    
    // Start the service
    const cwd = path.join(process.cwd(), service.dev.cwd);
    startService(service.name, service.dev.command, args, cwd, env);
    
    // Wait if specified
    if (service.dev.waitAfterStart) {
      console.log(`\n⏳ Waiting for ${service.name} to initialize...`);
      await new Promise(resolve => setTimeout(resolve, service.dev.waitAfterStart));
    }
  }
  
  // Success message
  console.log('\n╔════════════════════════════════════════╗');
  console.log('║        🎉 All Services Running!       ║');
  console.log('╚════════════════════════════════════════╝\n');
  console.log('📍 Access URLs:');
  for (const [key, port] of Object.entries(ports)) {
    const service = SERVICES[key];
    if (service.type === 'frontend') {
      console.log(`   ${service.name.padEnd(18)} http://localhost:${port}`);
    } else if (service.type === 'backend') {
      console.log(`   ${service.name.padEnd(18)} http://localhost:${port}/v1/health`);
    }
  }
  console.log('\n💡 Press Ctrl+C to stop all services\n');
}

// Cleanup on exit
process.on('SIGINT', () => {
  cleanup();
  process.exit(0);
});

process.on('SIGTERM', () => {
  cleanup();
  process.exit(0);
});

process.on('exit', cleanup);

// Run
main().catch((err) => {
  console.error('❌ Fatal error:', err);
  cleanup();
  process.exit(1);
});

