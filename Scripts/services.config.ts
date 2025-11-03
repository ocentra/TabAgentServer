/**
 * TabAgent Services Configuration
 * 
 * Single source of truth for all services in the system.
 * Add/remove services here - everything else auto-updates!
 */

export interface PortConfig {
  preferred: number;
  rangeStart: number;
  rangeEnd: number;
}

export interface DevConfig {
  command: string;
  args: string[];
  cwd: string;
  env?: Record<string, string>;
  waitAfterStart?: number;
}

export interface BuildConfig {
  command: string;
  args: string[];
  cwd: string;
}

export interface ServiceConfig {
  name: string;
  type: 'backend' | 'frontend';
  port: PortConfig;
  processNames: string[];
  dev?: DevConfig;
  build?: BuildConfig;
  proxy?: Record<string, string>;
}

export type ServicesConfig = Record<string, ServiceConfig>;

export const SERVICES: ServicesConfig = {
  // Backend services
  rust: {
    name: 'Rust Backend',
    type: 'backend',
    
    port: {
      preferred: 3000,
      rangeStart: 3000,
      rangeEnd: 4000
    },
    
    processNames: [
      'tabagent-server',
      'cargo',
      'target\\debug\\tabagent-server',
      'target\\release\\tabagent-server'
    ],
    
    dev: {
      command: 'cargo',
      args: ['run', '--bin', 'tabagent-server', '--', '--mode', 'http', '--port', '{port}'],
      cwd: 'Rust',
      env: {
        TABAGENT_PORT: '{port}'
      },
      waitAfterStart: 3000
    },
    
    build: {
      command: 'cargo',
      args: ['build', '--release'],
      cwd: 'Rust'
    }
  },

  dashboard: {
    name: 'Dashboard',
    type: 'frontend',
    
    port: {
      preferred: 5173,
      rangeStart: 5173,
      rangeEnd: 6000
    },
    
    processNames: ['vite', 'node'],
    
    dev: {
      command: 'npm',
      args: ['run', 'dev'],
      cwd: 'dashboard',
      env: {
        VITE_PORT: '{port}',
        VITE_RUST_PORT: '{rustPort}'
      }
    },
    
    build: {
      command: 'npm',
      args: ['run', 'build'],
      cwd: 'dashboard'
    },
    
    proxy: {
      '/v1': 'rust',
      '/ws': 'rust'
    }
  },

  agentBuilder: {
    name: 'Agent Builder',
    type: 'frontend',
    
    port: {
      preferred: 5175,
      rangeStart: 5175,
      rangeEnd: 6000
    },
    
    processNames: ['vite', 'node'],
    
    dev: {
      command: 'npm',
      args: ['run', 'dev'],
      cwd: 'agent-builder',
      env: {
        VITE_PORT: '{port}',
        VITE_RUST_PORT: '{rustPort}'
      }
    },
    
    build: {
      command: 'npm',
      args: ['run', 'build'],
      cwd: 'agent-builder'
    },
    
    proxy: {
      '/api': 'rust'
    }
  }
};

/**
 * Get service startup order (backends first, then frontends)
 */
export function getStartupOrder(): string[] {
  const backends: string[] = [];
  const frontends: string[] = [];
  
  for (const [key, config] of Object.entries(SERVICES)) {
    if (config.type === 'backend') {
      backends.push(key);
    } else if (config.type === 'frontend') {
      frontends.push(key);
    }
  }
  
  return [...backends, ...frontends];
}

/**
 * Resolve port placeholders in command/env
 */
export function resolvePortPlaceholders(value: string, allocatedPorts: Record<string, number>): string {
  let resolved = value;
  
  for (const [serviceKey, port] of Object.entries(allocatedPorts)) {
    resolved = resolved.replace(`{${serviceKey}Port}`, port.toString());
  }
  
  resolved = resolved.replace('{port}', '{CURRENT_PORT}');
  
  return resolved;
}

export default SERVICES;

