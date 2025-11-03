#!/usr/bin/env node

/**
 * Port Manager - Smart port allocation for TabAgent
 * 
 * Features:
 * - Single instance enforcement
 * - Auto-kill stale processes
 * - Friendly error on external conflicts
 * - Fallback port selection
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import process from 'process';
import net from 'net';
import fs from 'fs';
import path from 'path';
import { SERVICES, type ServiceConfig } from './services.config.js';

const execAsync = promisify(exec);

interface PortConfigExtended {
  name: string;
  preferred: number;
  rangeStart: number;
  rangeEnd: number;
  processNames: string[];
}

interface PortOccupant {
  pid: number;
  name: string;
  isOurs: boolean;
}

interface LockData {
  timestamp: number;
  pid: number;
  ports: Record<string, number>;
}

// Build port configuration from services config
const PORTS: Record<string, PortConfigExtended> = {};
for (const [key, service] of Object.entries(SERVICES)) {
  PORTS[key] = {
    name: service.name,
    preferred: service.port.preferred,
    rangeStart: service.port.rangeStart,
    rangeEnd: service.port.rangeEnd,
    processNames: service.processNames
  };
}

// Lock file path
const LOCK_FILE = path.join(process.cwd(), '.tabagent.lock');

/**
 * Check if a port is available
 */
async function isPortAvailable(port: number): Promise<boolean> {
  return new Promise((resolve) => {
    const server = net.createServer();
    
    server.once('error', () => {
      resolve(false);
    });
    
    server.once('listening', () => {
      server.close();
      resolve(true);
    });
    
    server.listen(port, '127.0.0.1');
  });
}

/**
 * Get process using a port (cross-platform)
 */
async function getPortOccupant(port: number): Promise<PortOccupant | null> {
  try {
    const isWindows = process.platform === 'win32';
    
    if (isWindows) {
      const { stdout } = await execAsync(`netstat -ano | findstr :${port}`);
      const lines = stdout.trim().split('\n');
      
      for (const line of lines) {
        const match = line.match(/LISTENING\s+(\d+)/);
        if (match) {
          const pid = match[1];
          const { stdout: taskList } = await execAsync(`tasklist /FI "PID eq ${pid}" /FO CSV /NH`);
          const processName = taskList.split(',')[0].replace(/"/g, '').trim();
          
          // Check if process is ours
          const isOurs = Object.values(PORTS).some(config =>
            config.processNames.some(n => processName.toLowerCase().includes(n.toLowerCase()))
          );
          
          return {
            pid: parseInt(pid),
            name: processName,
            isOurs
          };
        }
      }
    } else {
      const { stdout } = await execAsync(`lsof -i :${port} -t`);
      const pid = parseInt(stdout.trim());
      
      if (pid) {
        const { stdout: psOut } = await execAsync(`ps -p ${pid} -o comm=`);
        const processName = psOut.trim();
        
        const isOurs = Object.values(PORTS).some(config =>
          config.processNames.some(n => processName.includes(n))
        );
        
        return { pid, name: processName, isOurs };
      }
    }
  } catch (err) {
    return null;
  }
  
  return null;
}

/**
 * Kill a process by PID
 */
async function killProcess(pid: number): Promise<boolean> {
  try {
    const isWindows = process.platform === 'win32';
    
    if (isWindows) {
      await execAsync(`taskkill /F /PID ${pid}`);
    } else {
      await execAsync(`kill -9 ${pid}`);
    }
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    return true;
  } catch (err) {
    console.error(`Failed to kill process ${pid}:`, (err as Error).message);
    return false;
  }
}

/**
 * Find an available port from preferred or any port in range
 */
async function findAvailablePort(config: PortConfigExtended): Promise<number | null> {
  console.log(`  Checking port ${config.preferred}...`);
  
  const preferredAvailable = await isPortAvailable(config.preferred);
  
  if (preferredAvailable) {
    console.log(`  ✅ Port ${config.preferred} available`);
    return config.preferred;
  }
  
  const occupant = await getPortOccupant(config.preferred);
  
  if (occupant?.isOurs) {
    console.log(`  🔄 Killing stale process: ${occupant.name} (PID: ${occupant.pid})`);
    const killed = await killProcess(occupant.pid);
    
    if (killed && await isPortAvailable(config.preferred)) {
      console.log(`  ✅ Port ${config.preferred} now available`);
      return config.preferred;
    }
  }
  
  if (occupant && !occupant.isOurs) {
    console.log(`  ⚠️  Port ${config.preferred} in use by: ${occupant.name} (PID: ${occupant.pid})`);
  }
  
  console.log(`  🔍 Scanning for free port in range ${config.rangeStart}-${config.rangeEnd}...`);
  
  for (let port = config.rangeStart; port <= config.rangeEnd; port++) {
    if (port === config.preferred) continue;
    
    if (await isPortAvailable(port)) {
      console.log(`  ✅ Found free port: ${port}`);
      return port;
    }
  }
  
  return null;
}

/**
 * Check if process is alive
 */
function isProcessAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/**
 * Check for existing instance via lock file and kill if found
 */
async function checkSingleInstance(): Promise<void> {
  if (fs.existsSync(LOCK_FILE)) {
    try {
      const lockData: LockData = JSON.parse(fs.readFileSync(LOCK_FILE, 'utf8'));
      const oldPid = lockData.pid;
      
      if (isProcessAlive(oldPid)) {
        console.log(`🔄 Found existing TabAgent instance (PID: ${oldPid}), killing it...`);
        
        const killed = await killProcess(oldPid);
        
        if (killed) {
          console.log('   ✅ Old instance killed successfully');
        } else {
          console.log('   ⚠️  Failed to kill old instance, continuing anyway...');
        }
      } else {
        console.log('⚠️  Stale lock file detected (process not running), removing...');
      }
      
      fs.unlinkSync(LOCK_FILE);
    } catch {
      console.log('⚠️  Invalid lock file, removing...');
      fs.unlinkSync(LOCK_FILE);
    }
  }
}

/**
 * Create lock file
 */
function createLockFile(ports: Record<string, number>): void {
  const lockData: LockData = {
    timestamp: Date.now(),
    pid: process.pid,
    ports
  };
  
  fs.writeFileSync(LOCK_FILE, JSON.stringify(lockData, null, 2));
  
  const cleanup = () => {
    try {
      fs.unlinkSync(LOCK_FILE);
    } catch {
      // Ignore
    }
  };
  
  process.on('exit', cleanup);
  process.on('SIGINT', () => {
    cleanup();
    process.exit(0);
  });
}

/**
 * Main port allocation logic
 */
export async function allocatePorts(): Promise<Record<string, number>> {
  console.log('\n🔍 TabAgent Port Manager\n');
  
  console.log('1️⃣  Checking for existing instance...');
  await checkSingleInstance();
  console.log('   ✅ No other instances detected\n');
  
  console.log('2️⃣  Allocating ports...\n');
  
  const allocatedPorts: Record<string, number> = {};
  
  for (const [key, config] of Object.entries(PORTS)) {
    console.log(`📍 ${config.name}:`);
    const port = await findAvailablePort(config);
    
    if (!port) {
      console.error(`\n❌ ERROR: No available ports for ${config.name}`);
      console.error(`   Tried range: ${config.rangeStart}-${config.rangeEnd}`);
      console.error(`   All ports are busy! This should never happen.`);
      console.error(`   Please check your system and try again.\n`);
      process.exit(1);
    }
    
    allocatedPorts[key] = port;
    console.log('');
  }
  
  createLockFile(allocatedPorts);
  
  console.log('✅ All ports allocated successfully!\n');
  console.log('📋 Port Assignments:');
  console.log(`   Rust Backend:    http://localhost:${allocatedPorts.rust}`);
  console.log(`   Dashboard:       http://localhost:${allocatedPorts.dashboard}`);
  console.log(`   Agent Builder:   http://localhost:${allocatedPorts.agentBuilder}`);
  console.log('');
  
  process.env.TABAGENT_RUST_PORT = allocatedPorts.rust.toString();
  process.env.TABAGENT_DASHBOARD_PORT = allocatedPorts.dashboard.toString();
  process.env.TABAGENT_BUILDER_PORT = allocatedPorts.agentBuilder.toString();
  
  return allocatedPorts;
}

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  allocatePorts()
    .then(() => {
      console.log('🚀 Ready to start services!\n');
      process.exit(0);
    })
    .catch((err) => {
      console.error('❌ Fatal error:', (err as Error).message);
      process.exit(1);
    });
}

export { isPortAvailable, getPortOccupant, killProcess };

