import React, { useState, useRef, useCallback, useEffect, Suspense } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text, Html, PerspectiveCamera } from '@react-three/drei';
import * as THREE from 'three';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

interface GraphNode {
  id: string;
  label: string;
  type: 'entity' | 'concept' | 'document' | 'relationship' | 'person' | 'organization' | 'location' | 'event';
  x: number;
  y: number;
  size: number;
  color: string;
  connections: string[];
  metadata: {
    confidence: number;
    source: string;
    created: Date;
    weight: number;
    description?: string;
    tags: string[];
    lastAccessed?: Date;
  };
  cluster?: string;
  importance: number;
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  relationship: string;
  strength: number;
  bidirectional: boolean;
  color: string;
  width: number;
}

interface GraphCluster {
  id: string;
  label: string;
  color: string;
  nodes: string[];
  center: { x: number; y: number };
  radius: number;
}

// Layout algorithms
const applyForceLayout = (nodes: GraphNode[], edges: GraphEdge[]) => {
  // Simple force-directed layout simulation with edge-based positioning
  const iterations = 50;
  let newNodes = [...nodes];
  
  for (let i = 0; i < iterations; i++) {
    newNodes = newNodes.map(node => {
      let fx = 0, fy = 0;
      
      // Repulsion from other nodes
      newNodes.forEach(otherNode => {
        if (otherNode.id !== node.id) {
          const dx = node.x - otherNode.x;
          const dy = node.y - otherNode.y;
          const distance = Math.sqrt(dx * dx + dy * dy) || 1;
          const force = 1000 / (distance * distance);
          fx += (dx / distance) * force;
          fy += (dy / distance) * force;
        }
      });
      
      // Attraction along edges
      edges.forEach(edge => {
        if (edge.source === node.id || edge.target === node.id) {
          const otherId = edge.source === node.id ? edge.target : edge.source;
          const otherNode = newNodes.find(n => n.id === otherId);
          if (otherNode) {
            const dx = otherNode.x - node.x;
            const dy = otherNode.y - node.y;
            const distance = Math.sqrt(dx * dx + dy * dy) || 1;
            const force = distance * 0.01;
            fx += (dx / distance) * force;
            fy += (dy / distance) * force;
          }
        }
      });
      
      return {
        ...node,
        x: node.x + fx * 0.1,
        y: node.y + fy * 0.1
      };
    });
  }
  
  return newNodes;
};

const applyHierarchicalLayout = (nodes: GraphNode[], _edges: GraphEdge[], rootNodeId?: string) => {
  const levels: { [key: number]: GraphNode[] } = {};
  const visited = new Set<string>();
  const nodeDepths: { [key: string]: number } = {};
  
  // Find root node or use the most connected one
  const rootNode = rootNodeId ? nodes.find(n => n.id === rootNodeId) : 
    nodes.reduce((prev, current) => (prev.connections.length > current.connections.length) ? prev : current);
  
  if (!rootNode) return nodes;
  
  // BFS to assign levels
  const queue = [{ node: rootNode, depth: 0 }];
  visited.add(rootNode.id);
  nodeDepths[rootNode.id] = 0;
  
  while (queue.length > 0) {
    const { node, depth } = queue.shift()!;
    
    if (!levels[depth]) levels[depth] = [];
    levels[depth].push(node);
    
    node.connections.forEach(connId => {
      if (!visited.has(connId)) {
        const connectedNode = nodes.find(n => n.id === connId);
        if (connectedNode) {
          visited.add(connId);
          nodeDepths[connId] = depth + 1;
          queue.push({ node: connectedNode, depth: depth + 1 });
        }
      }
    });
  }
  
  // Position nodes in hierarchy
  const levelHeight = 120;
  const nodeSpacing = 150;
  
  return nodes.map(node => {
    const depth = nodeDepths[node.id] || 0;
    const levelNodes = levels[depth] || [];
    const nodeIndex = levelNodes.findIndex(n => n.id === node.id);
    const levelWidth = levelNodes.length * nodeSpacing;
    
    return {
      ...node,
      x: (nodeIndex * nodeSpacing) - (levelWidth / 2) + 400,
      y: depth * levelHeight + 100
    };
  });
};

const applyCircularLayout = (nodes: GraphNode[], _edges: GraphEdge[]) => {
  const centerX = 400;
  const centerY = 300;
  
  // Group nodes by type for better circular arrangement
  const nodesByType = nodes.reduce((acc, node) => {
    if (!acc[node.type]) acc[node.type] = [];
    acc[node.type].push(node);
    return acc;
  }, {} as Record<string, GraphNode[]>);
  
  const types = Object.keys(nodesByType);
  
  return nodes.map(node => {
    const typeIndex = types.indexOf(node.type);
    const nodesInType = nodesByType[node.type].length;
    const nodeInTypeIndex = nodesByType[node.type].findIndex(n => n.id === node.id);
    
    // Create concentric circles for different types
    const radius = 150 + (typeIndex * 80);
    const angle = (nodeInTypeIndex / nodesInType) * 2 * Math.PI;
    
    return {
      ...node,
      x: centerX + Math.cos(angle) * radius,
      y: centerY + Math.sin(angle) * radius
    };
  });
};

// Enterprise-grade mock knowledge graph data
const mockClusters: GraphCluster[] = [
  { id: 'ai-ml', label: 'AI & Machine Learning', color: '#3b82f6', nodes: [], center: { x: 200, y: 150 }, radius: 120 },
  { id: 'data-science', label: 'Data Science', color: '#10b981', nodes: [], center: { x: 500, y: 200 }, radius: 100 },
  { id: 'nlp', label: 'Natural Language Processing', color: '#f59e0b', nodes: [], center: { x: 350, y: 400 }, radius: 90 },
  { id: 'computer-vision', label: 'Computer Vision', color: '#ef4444', nodes: [], center: { x: 150, y: 350 }, radius: 85 },
  { id: 'research', label: 'Research & Papers', color: '#8b5cf6', nodes: [], center: { x: 600, y: 100 }, radius: 70 }
];

const mockNodes: GraphNode[] = [
  // AI & ML Cluster
  {
    id: '1', label: 'Machine Learning', type: 'concept', x: 200, y: 150, size: 25, color: '#3b82f6',
    connections: ['2', '3', '4', '5'], cluster: 'ai-ml', importance: 0.95,
    metadata: { confidence: 0.95, source: 'AI Research Compendium.pdf', created: new Date('2024-01-15'), weight: 10, 
                description: 'Core field of artificial intelligence focused on algorithms that learn from data',
                tags: ['AI', 'algorithms', 'learning'], lastAccessed: new Date() }
  },
  {
    id: '2', label: 'Neural Networks', type: 'concept', x: 180, y: 120, size: 22, color: '#3b82f6',
    connections: ['1', '6', '7', '8'], cluster: 'ai-ml', importance: 0.92,
    metadata: { confidence: 0.92, source: 'Deep Learning Fundamentals.pdf', created: new Date('2024-01-10'), weight: 9,
                description: 'Computing systems inspired by biological neural networks',
                tags: ['neural', 'deep-learning', 'networks'], lastAccessed: new Date() }
  },
  {
    id: '3', label: 'Supervised Learning', type: 'concept', x: 220, y: 180, size: 18, color: '#3b82f6',
    connections: ['1', '9'], cluster: 'ai-ml', importance: 0.88,
    metadata: { confidence: 0.88, source: 'ML Textbook Chapter 3.pdf', created: new Date('2024-01-08'), weight: 7,
                description: 'Learning with labeled training data',
                tags: ['supervised', 'training', 'labels'] }
  },
  {
    id: '4', label: 'Deep Learning', type: 'concept', x: 160, y: 180, size: 24, color: '#3b82f6',
    connections: ['1', '2', '6', '10'], cluster: 'ai-ml', importance: 0.94,
    metadata: { confidence: 0.94, source: 'Deep Learning Research.pdf', created: new Date('2024-01-12'), weight: 9,
                description: 'Machine learning using deep neural networks',
                tags: ['deep', 'neural', 'layers'] }
  },
  
  // Data Science Cluster
  {
    id: '5', label: 'Data Analysis', type: 'concept', x: 500, y: 200, size: 20, color: '#10b981',
    connections: ['1', '11', '12'], cluster: 'data-science', importance: 0.85,
    metadata: { confidence: 0.85, source: 'Data Science Handbook.pdf', created: new Date('2024-01-05'), weight: 8,
                description: 'Process of inspecting and modeling data',
                tags: ['data', 'analysis', 'statistics'] }
  },
  {
    id: '11', label: 'Statistical Modeling', type: 'concept', x: 480, y: 170, size: 17, color: '#10b981',
    connections: ['5', '12'], cluster: 'data-science', importance: 0.78,
    metadata: { confidence: 0.78, source: 'Statistics for Data Science.pdf', created: new Date('2024-01-03'), weight: 6,
                description: 'Mathematical models for data relationships',
                tags: ['statistics', 'modeling', 'mathematics'] }
  },
  {
    id: '12', label: 'Data Visualization', type: 'concept', x: 520, y: 230, size: 19, color: '#10b981',
    connections: ['5', '11'], cluster: 'data-science', importance: 0.82,
    metadata: { confidence: 0.82, source: 'Visualization Principles.pdf', created: new Date('2024-01-07'), weight: 7,
                description: 'Graphical representation of data and information',
                tags: ['visualization', 'charts', 'graphics'] }
  },
  
  // NLP Cluster
  {
    id: '6', label: 'Natural Language Processing', type: 'concept', x: 350, y: 400, size: 23, color: '#f59e0b',
    connections: ['2', '4', '13', '14'], cluster: 'nlp', importance: 0.91,
    metadata: { confidence: 0.91, source: 'NLP Comprehensive Guide.pdf', created: new Date('2024-01-14'), weight: 9,
                description: 'AI field focused on human language understanding',
                tags: ['NLP', 'language', 'text'] }
  },
  {
    id: '13', label: 'Transformers', type: 'entity', x: 330, y: 370, size: 21, color: '#f59e0b',
    connections: ['6', '14'], cluster: 'nlp', importance: 0.89,
    metadata: { confidence: 0.89, source: 'Attention Is All You Need.pdf', created: new Date('2024-01-11'), weight: 8,
                description: 'Revolutionary neural network architecture for NLP',
                tags: ['transformers', 'attention', 'BERT'] }
  },
  {
    id: '14', label: 'Large Language Models', type: 'entity', x: 370, y: 430, size: 26, color: '#f59e0b',
    connections: ['6', '13'], cluster: 'nlp', importance: 0.96,
    metadata: { confidence: 0.96, source: 'LLM Survey Paper.pdf', created: new Date('2024-01-16'), weight: 10,
                description: 'Massive neural networks trained on text data',
                tags: ['LLM', 'GPT', 'language-models'] }
  },
  
  // Computer Vision Cluster
  {
    id: '7', label: 'Computer Vision', type: 'concept', x: 150, y: 350, size: 22, color: '#ef4444',
    connections: ['2', '8', '15'], cluster: 'computer-vision', importance: 0.87,
    metadata: { confidence: 0.87, source: 'Computer Vision Textbook.pdf', created: new Date('2024-01-09'), weight: 8,
                description: 'AI field for interpreting visual information',
                tags: ['vision', 'images', 'recognition'] }
  },
  {
    id: '8', label: 'Convolutional Networks', type: 'entity', x: 130, y: 320, size: 20, color: '#ef4444',
    connections: ['2', '7', '15'], cluster: 'computer-vision', importance: 0.84,
    metadata: { confidence: 0.84, source: 'CNN Architecture Guide.pdf', created: new Date('2024-01-06'), weight: 7,
                description: 'Neural networks specialized for image processing',
                tags: ['CNN', 'convolution', 'images'] }
  },
  {
    id: '15', label: 'Object Detection', type: 'entity', x: 170, y: 380, size: 18, color: '#ef4444',
    connections: ['7', '8'], cluster: 'computer-vision', importance: 0.81,
    metadata: { confidence: 0.81, source: 'Object Detection Survey.pdf', created: new Date('2024-01-04'), weight: 6,
                description: 'Identifying and locating objects in images',
                tags: ['detection', 'objects', 'bounding-boxes'] }
  },
  
  // Research Cluster
  {
    id: '9', label: 'Geoffrey Hinton', type: 'person', x: 600, y: 100, size: 16, color: '#8b5cf6',
    connections: ['3', '10'], cluster: 'research', importance: 0.93,
    metadata: { confidence: 0.93, source: 'AI Pioneers Biography.pdf', created: new Date('2024-01-13'), weight: 9,
                description: 'Pioneer in artificial neural networks and deep learning',
                tags: ['researcher', 'neural-networks', 'pioneer'] }
  },
  {
    id: '10', label: 'Backpropagation Algorithm', type: 'entity', x: 580, y: 130, size: 19, color: '#8b5cf6',
    connections: ['4', '9'], cluster: 'research', importance: 0.86,
    metadata: { confidence: 0.86, source: 'Backpropagation Paper.pdf', created: new Date('2024-01-02'), weight: 7,
                description: 'Algorithm for training neural networks',
                tags: ['algorithm', 'training', 'gradient-descent'] }
  }
];

const mockEdges: GraphEdge[] = [
  { id: 'e1', source: '1', target: '2', relationship: 'includes', strength: 0.9, bidirectional: false, color: '#64748b', width: 3 },
  { id: 'e2', source: '1', target: '3', relationship: 'type_of', strength: 0.8, bidirectional: false, color: '#64748b', width: 2 },
  { id: 'e3', source: '1', target: '4', relationship: 'related_to', strength: 0.95, bidirectional: true, color: '#3b82f6', width: 4 },
  { id: 'e4', source: '2', target: '6', relationship: 'enables', strength: 0.85, bidirectional: false, color: '#64748b', width: 2 },
  { id: 'e5', source: '2', target: '7', relationship: 'enables', strength: 0.82, bidirectional: false, color: '#64748b', width: 2 },
  { id: 'e6', source: '2', target: '8', relationship: 'implements', strength: 0.88, bidirectional: false, color: '#64748b', width: 3 },
  { id: 'e7', source: '4', target: '6', relationship: 'powers', strength: 0.91, bidirectional: false, color: '#f59e0b', width: 3 },
  { id: 'e8', source: '6', target: '13', relationship: 'uses', strength: 0.87, bidirectional: false, color: '#f59e0b', width: 2 },
  { id: 'e9', source: '13', target: '14', relationship: 'evolved_into', strength: 0.93, bidirectional: false, color: '#f59e0b', width: 4 },
  { id: 'e10', source: '7', target: '8', relationship: 'uses', strength: 0.89, bidirectional: false, color: '#ef4444', width: 3 },
  { id: 'e11', source: '8', target: '15', relationship: 'enables', strength: 0.84, bidirectional: false, color: '#ef4444', width: 2 },
  { id: 'e12', source: '1', target: '5', relationship: 'applied_in', strength: 0.78, bidirectional: false, color: '#64748b', width: 2 },
  { id: 'e13', source: '5', target: '11', relationship: 'uses', strength: 0.75, bidirectional: false, color: '#10b981', width: 2 },
  { id: 'e14', source: '5', target: '12', relationship: 'produces', strength: 0.80, bidirectional: false, color: '#10b981', width: 2 },
  { id: 'e15', source: '3', target: '9', relationship: 'researched_by', strength: 0.85, bidirectional: false, color: '#64748b', width: 2 },
  { id: 'e16', source: '4', target: '10', relationship: 'uses', strength: 0.92, bidirectional: false, color: '#64748b', width: 3 }
];

// 3D Node Component
const Node3D: React.FC<{
  node: GraphNode;
  isSelected: boolean;
  isFocused: boolean;
  onSelect: (id: string) => void;
}> = ({ node, isSelected, isFocused, onSelect }) => {
  const meshRef = useRef<THREE.Mesh>(null);
  const [hovered, setHovered] = useState(false);

  useFrame(() => {
    if (meshRef.current) {
      meshRef.current.rotation.y += 0.01;
      const targetScale = (isSelected || isFocused) ? 1.5 : (hovered ? 1.2 : 1);
      meshRef.current.scale.lerp(new THREE.Vector3(targetScale, targetScale, targetScale), 0.1);
    }
  });

  const getNodeGeometry = (type: string) => {
    switch (type) {
      case 'concept': return <sphereGeometry args={[1, 16, 16]} />;
      case 'entity': return <boxGeometry args={[1.5, 1.5, 1.5]} />;
      case 'person': return <octahedronGeometry args={[1.2]} />;
      case 'document': return <cylinderGeometry args={[1, 1, 2, 8]} />;
      default: return <sphereGeometry args={[1, 8, 8]} />;
    }
  };

  const position: [number, number, number] = [
    (node.x - 400) / 50, 
    -(node.y - 300) / 50, 
    0
  ];

  return (
    <group position={position}>
      <mesh
        ref={meshRef}
        onClick={() => onSelect(node.id)}
        onPointerOver={() => setHovered(true)}
        onPointerOut={() => setHovered(false)}
      >
        {getNodeGeometry(node.type)}
        <meshStandardMaterial 
          color={node.color}
          transparent
          opacity={isSelected || isFocused ? 1 : 0.8}
          emissive={hovered ? node.color : '#000000'}
          emissiveIntensity={hovered ? 0.3 : 0}
        />
      </mesh>
      
      <Text
        position={[0, 2.5, 0]}
        fontSize={0.8}
        color={isSelected || isFocused ? '#ffffff' : '#cccccc'}
        anchorX="center"
        anchorY="middle"
      >
        {node.label}
      </Text>
      
      {(isSelected || isFocused) && (
        <Html position={[0, -3, 0]} center>
          <div className="bg-card border border-border rounded-lg p-3 text-xs max-w-64 shadow-lg">
            <div className="font-semibold text-sm mb-1">{node.label}</div>
            <div className="text-muted-foreground space-y-1">
              <div>Type: <span className="capitalize">{node.type}</span></div>
              <div>Confidence: {(node.metadata.confidence * 100).toFixed(1)}%</div>
              <div>Connections: {node.connections.length}</div>
              <div>Importance: {(node.importance * 10).toFixed(1)}/10</div>
            </div>
          </div>
        </Html>
      )}
    </group>
  );
};

// 3D Edge Component
const Edge3D: React.FC<{ edge: GraphEdge; nodes: GraphNode[] }> = ({ edge, nodes }) => {
  const sourceNode = nodes.find(n => n.id === edge.source);
  const targetNode = nodes.find(n => n.id === edge.target);
  
  if (!sourceNode || !targetNode) return null;

  const start = new THREE.Vector3(
    (sourceNode.x - 400) / 50, 
    -(sourceNode.y - 300) / 50, 
    0
  );
  const end = new THREE.Vector3(
    (targetNode.x - 400) / 50, 
    -(targetNode.y - 300) / 50, 
    0
  );
  
  const points = [start, end];
  
  return (
    <line>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          args={[new Float32Array(points.flatMap(p => [p.x, p.y, p.z])), 3]}
        />
      </bufferGeometry>
      <lineBasicMaterial 
        color={edge.color} 
        transparent 
        opacity={edge.strength * 0.8} 
      />
    </line>
  );
};

// 3D Scene Component
const Scene3D: React.FC<{
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNode: string | null;
  focusedNode: string | null;
  onNodeSelect: (id: string) => void;
}> = ({ nodes, edges, selectedNode, focusedNode, onNodeSelect }) => {
  return (
    <>
      <PerspectiveCamera makeDefault position={[15, 10, 15]} />
      <OrbitControls enablePan enableZoom enableRotate />
      
      <ambientLight intensity={0.4} />
      <pointLight position={[10, 10, 10]} intensity={1} />
      <pointLight position={[-10, -10, -10]} intensity={0.5} />
      <directionalLight position={[0, 10, 5]} intensity={0.5} />
      
      {/* Render edges */}
      {edges.map(edge => (
        <Edge3D key={edge.id} edge={edge} nodes={nodes} />
      ))}
      
      {/* Render nodes */}
      {nodes.map(node => (
        <Node3D
          key={node.id}
          node={node}
          isSelected={selectedNode === node.id}
          isFocused={focusedNode === node.id}
          onSelect={onNodeSelect}
        />
      ))}
      
      {/* Grid helper */}
      <gridHelper args={[30, 30, '#333333', '#111111']} />
    </>
  );
};

// Enterprise-grade SVG Graph Visualization Component
const EnterpriseGraphVisualization: React.FC<{
  nodes: GraphNode[];
  edges: GraphEdge[];
  clusters: GraphCluster[];
  selectedNode: string | null;
  hoveredNode: string | null;
  onNodeSelect: (id: string) => void;
  onNodeHover: (id: string | null) => void;
  width: number;
  height: number;
  zoom: number;
  pan: { x: number; y: number };
  onZoomChange: (zoom: number) => void;
  onPanChange: (pan: { x: number; y: number }) => void;
}> = ({ nodes, edges, clusters, selectedNode, hoveredNode, onNodeSelect, onNodeHover, width, height, zoom, pan, onZoomChange, onPanChange }) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Handle mouse wheel for zooming
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(5, zoom * zoomFactor));
    onZoomChange(newZoom);
  }, [zoom, onZoomChange]);

  // Handle mouse drag for panning
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.target === svgRef.current) {
      setIsDragging(true);
      setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    }
  }, [pan]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (isDragging) {
      onPanChange({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y
      });
    }
  }, [isDragging, dragStart, onPanChange]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // Update cluster positions based on layout
  const getUpdatedClusters = useCallback(() => {
    return clusters.map(cluster => {
      const clusterNodes = nodes.filter(n => n.cluster === cluster.id);
      if (clusterNodes.length === 0) return cluster;

      // Calculate center based on actual node positions
      const centerX = clusterNodes.reduce((sum, node) => sum + node.x, 0) / clusterNodes.length;
      const centerY = clusterNodes.reduce((sum, node) => sum + node.y, 0) / clusterNodes.length;
      
      // Calculate radius based on node spread
      const maxDistance = Math.max(...clusterNodes.map(node => 
        Math.sqrt((node.x - centerX) ** 2 + (node.y - centerY) ** 2)
      ));
      
      return {
        ...cluster,
        center: { x: centerX, y: centerY },
        radius: Math.max(maxDistance + 50, 80)
      };
    });
  }, [clusters, nodes]);
  
  const getNodeIcon = (type: string) => {
    switch (type) {
      case 'concept':
        return (
          <g>
            <circle r="8" fill="currentColor" opacity="0.2" />
            <circle r="6" fill="currentColor" />
            <circle r="3" fill="white" opacity="0.8" />
          </g>
        );
      case 'entity':
        return (
          <g>
            <rect x="-8" y="-8" width="16" height="16" fill="currentColor" opacity="0.2" rx="2" />
            <rect x="-6" y="-6" width="12" height="12" fill="currentColor" rx="1" />
            <rect x="-3" y="-3" width="6" height="6" fill="white" opacity="0.8" />
          </g>
        );
      case 'person':
        return (
          <g>
            <circle r="8" fill="currentColor" opacity="0.2" />
            <circle r="6" fill="currentColor" />
            <circle cx="0" cy="-2" r="2" fill="white" />
            <path d="M -3 2 Q 0 0 3 2" stroke="white" strokeWidth="1.5" fill="none" />
          </g>
        );
      case 'document':
        return (
          <g>
            <rect x="-7" y="-9" width="14" height="18" fill="currentColor" opacity="0.2" rx="1" />
            <rect x="-5" y="-7" width="10" height="14" fill="currentColor" rx="1" />
            <line x1="-3" y1="-4" x2="3" y2="-4" stroke="white" strokeWidth="1" />
            <line x1="-3" y1="-1" x2="3" y2="-1" stroke="white" strokeWidth="1" />
            <line x1="-3" y1="2" x2="1" y2="2" stroke="white" strokeWidth="1" />
          </g>
        );
      default:
        return (
          <g>
            <circle r="6" fill="currentColor" />
          </g>
        );
    }
  };

  const getEdgePath = (edge: GraphEdge) => {
    const sourceNode = nodes.find(n => n.id === edge.source);
    const targetNode = nodes.find(n => n.id === edge.target);
    
    if (!sourceNode || !targetNode) return '';
    
    const dx = targetNode.x - sourceNode.x;
    const dy = targetNode.y - sourceNode.y;
    const distance = Math.sqrt(dx * dx + dy * dy);
    
    // Create curved path for better visual appeal
    const midX = (sourceNode.x + targetNode.x) / 2;
    const midY = (sourceNode.y + targetNode.y) / 2;
    const curvature = Math.min(distance * 0.2, 50);
    const controlX = midX + (dy / distance) * curvature;
    const controlY = midY - (dx / distance) * curvature;
    
    return `M ${sourceNode.x} ${sourceNode.y} Q ${controlX} ${controlY} ${targetNode.x} ${targetNode.y}`;
  };

  const updatedClusters = getUpdatedClusters();

  return (
    <svg 
      ref={svgRef}
      width={width} 
      height={height} 
      className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 border border-border rounded-lg overflow-hidden"
      style={{ cursor: isDragging ? 'grabbing' : 'grab' }}
      onWheel={handleWheel}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      <defs>
        {/* Gradient definitions for enhanced visuals */}
        <radialGradient id="nodeGlow" cx="50%" cy="50%" r="50%">
          <stop offset="0%" stopColor="white" stopOpacity="0.8" />
          <stop offset="100%" stopColor="white" stopOpacity="0" />
        </radialGradient>
        
        {/* Arrow markers for directed edges */}
        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
          <polygon points="0 0, 10 3.5, 0 7" fill="#64748b" />
        </marker>
        
        {/* Drop shadow filter */}
        <filter id="dropshadow" x="-50%" y="-50%" width="200%" height="200%">
          <feDropShadow dx="2" dy="2" stdDeviation="3" floodOpacity="0.3" />
        </filter>
      </defs>
      
      <g transform={`translate(${pan.x}, ${pan.y}) scale(${zoom})`}>
        {/* Render cluster backgrounds - NO MORE BLINKING! */}
        {updatedClusters.map(cluster => (
          <g key={cluster.id}>
            <circle
              cx={cluster.center.x}
              cy={cluster.center.y}
              r={cluster.radius}
              fill={cluster.color}
              opacity="0.08"
              stroke={cluster.color}
              strokeWidth="1.5"
              strokeDasharray="8,4"
              className="transition-all duration-500"
            />
            <text
              x={cluster.center.x}
              y={cluster.center.y - cluster.radius - 15}
              textAnchor="middle"
              className="text-xs font-medium fill-current text-muted-foreground pointer-events-none"
            >
              {cluster.label}
            </text>
          </g>
        ))}
        
        {/* Render edges */}
        {edges.map(edge => {
          const isHighlighted = selectedNode === edge.source || selectedNode === edge.target ||
                               hoveredNode === edge.source || hoveredNode === edge.target;
          
          return (
            <g key={edge.id}>
              <path
                d={getEdgePath(edge)}
                stroke={isHighlighted ? edge.color : '#e2e8f0'}
                strokeWidth={isHighlighted ? edge.width : Math.max(1, edge.width - 1)}
                fill="none"
                opacity={isHighlighted ? 0.8 : 0.4}
                markerEnd="url(#arrowhead)"
                className="transition-all duration-300"
              />
              
              {/* Edge label */}
              {isHighlighted && (
                <text
                  x={(nodes.find(n => n.id === edge.source)!.x + nodes.find(n => n.id === edge.target)!.x) / 2}
                  y={(nodes.find(n => n.id === edge.source)!.y + nodes.find(n => n.id === edge.target)!.y) / 2}
                  textAnchor="middle"
                  className="text-xs fill-current text-muted-foreground bg-background px-1 rounded"
                  dy="-5"
                >
                  {edge.relationship}
                </text>
              )}
            </g>
          );
        })}
        
        {/* Render nodes */}
        {nodes.map(node => {
          const isSelected = selectedNode === node.id;
          const isHovered = hoveredNode === node.id;
          const isConnected = selectedNode && node.connections.includes(selectedNode);
          const shouldHighlight = isSelected || isHovered || isConnected;
          
          return (
            <g
              key={node.id}
              transform={`translate(${node.x}, ${node.y})`}
              className="cursor-pointer transition-all duration-300"
              onClick={() => onNodeSelect(node.id)}
              onMouseEnter={() => onNodeHover(node.id)}
              onMouseLeave={() => onNodeHover(null)}
            >
              {/* Node glow effect */}
              {shouldHighlight && (
                <circle
                  r={node.size + 8}
                  fill="url(#nodeGlow)"
                  opacity="0.6"
                  className="animate-pulse"
                />
              )}
              
              {/* Node shadow */}
              <g filter="url(#dropshadow)">
                <circle
                  r={node.size}
                  fill={node.color}
                  opacity={shouldHighlight ? 1 : 0.8}
                  stroke={isSelected ? '#ffffff' : 'transparent'}
                  strokeWidth={isSelected ? 3 : 0}
                />
              </g>
              
              {/* Node icon */}
              <g color={node.color} className="pointer-events-none">
                {getNodeIcon(node.type)}
              </g>
              
              {/* Node label */}
              <text
                y={node.size + 20}
                textAnchor="middle"
                className={`text-sm font-medium fill-current transition-all duration-300 ${
                  shouldHighlight ? 'text-foreground' : 'text-muted-foreground'
                }`}
                style={{ fontSize: shouldHighlight ? '14px' : '12px' }}
              >
                {node.label}
              </text>
              
              {/* Importance indicator */}
              <circle
                cx={node.size - 5}
                cy={-node.size + 5}
                r="4"
                fill={`hsl(${node.importance * 120}, 70%, 50%)`}
                opacity="0.8"
              />
              <text
                x={node.size - 5}
                y={-node.size + 8}
                textAnchor="middle"
                className="text-xs fill-white font-bold"
              >
                {Math.round(node.importance * 10)}
              </text>
            </g>
          );
        })}
      </g>
    </svg>
  );
};

const KnowledgeGraph: React.FC = () => {
  const [originalNodes] = useState<GraphNode[]>(mockNodes);
  const [nodes, setNodes] = useState<GraphNode[]>(mockNodes);
  const [edges] = useState<GraphEdge[]>(mockEdges);
  const [clusters] = useState<GraphCluster[]>(mockClusters);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [focusedNode, setFocusedNode] = useState<string | null>(null);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'concept' | 'entity' | 'document' | 'person'>('all');
  const [layoutMode, setLayoutMode] = useState<'force' | 'hierarchical' | 'circular'>('force');
  const [viewMode, setViewMode] = useState<'2d' | '3d'>('2d');
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 50, y: 50 });
  const [showClusters, setShowClusters] = useState(true);
  const [showLabels, setShowLabels] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedNodeData = selectedNode ? nodes.find(n => n.id === selectedNode) : null;

  const filteredNodes = nodes.filter(node => {
    const matchesSearch = node.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         node.metadata.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
    const matchesFilter = filterType === 'all' || node.type === filterType;
    return matchesSearch && matchesFilter;
  });

  const filteredEdges = edges.filter(edge => 
    filteredNodes.some(n => n.id === edge.source) && filteredNodes.some(n => n.id === edge.target)
  );

  // Apply layout when layout mode changes
  useEffect(() => {
    let newNodes: GraphNode[];
    
    switch (layoutMode) {
      case 'hierarchical':
        newNodes = applyHierarchicalLayout(originalNodes, edges, focusedNode || undefined);
        break;
      case 'circular':
        newNodes = applyCircularLayout(originalNodes, edges);
        break;
      case 'force':
      default:
        newNodes = applyForceLayout(originalNodes, edges);
        break;
    }
    
    setNodes(newNodes);
  }, [layoutMode, originalNodes, edges, focusedNode]);

  const handleNodeSelect = useCallback((nodeId: string) => {
    setSelectedNode(selectedNode === nodeId ? null : nodeId);
  }, [selectedNode]);

  const handleNodeHover = useCallback((nodeId: string | null) => {
    setHoveredNode(nodeId);
  }, []);

  const handleFocusNode = useCallback((nodeId: string) => {
    setFocusedNode(nodeId);
    setSelectedNode(nodeId);
    
    // If in hierarchical mode, re-layout with this node as root
    if (layoutMode === 'hierarchical') {
      const newNodes = applyHierarchicalLayout(originalNodes, edges, nodeId);
      setNodes(newNodes);
    }
  }, [layoutMode, originalNodes, edges]);

  const handleExpandNode = useCallback((nodeId: string) => {
    const node = nodes.find(n => n.id === nodeId);
    if (!node) return;
    
    // Show connected nodes with enhanced visibility
    const connectedNodeIds = new Set([nodeId, ...node.connections]);
    
    // Filter to show only connected subgraph
    const expandedNodes = nodes.map(n => ({
      ...n,
      size: connectedNodeIds.has(n.id) ? n.size * 1.5 : n.size * 0.7,
      color: connectedNodeIds.has(n.id) ? n.color : '#94a3b8'
    }));
    
    setNodes(expandedNodes);
    setFocusedNode(nodeId);
  }, [nodes]);

  const handleCollapseView = useCallback(() => {
    setNodes(originalNodes);
    setFocusedNode(null);
    setSelectedNode(null);
  }, [originalNodes]);

  const handleSearch = () => {
    // Advanced search functionality
    if (searchQuery.trim()) {
      const matchingNode = nodes.find(n => 
        n.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
        n.metadata.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      );
      
      if (matchingNode) {
        handleFocusNode(matchingNode.id);
      }
    }
  };

  const handleLayoutChange = useCallback((newLayout: 'force' | 'hierarchical' | 'circular') => {
    setLayoutMode(newLayout);
  }, []);

  const handleZoomChange = useCallback((newZoom: number) => {
    setZoom(newZoom);
  }, []);

  const handlePanChange = useCallback((newPan: { x: number; y: number }) => {
    setPan(newPan);
  }, []);

  const handleZoomIn = () => setZoom(prev => Math.min(prev * 1.2, 5));
  const handleZoomOut = () => setZoom(prev => Math.max(prev / 1.2, 0.1));
  const handleResetView = () => {
    setZoom(1);
    setPan({ x: 50, y: 50 });
    handleCollapseView();
  };

  // Get graph statistics
  const getGraphStats = () => {
    const totalNodes = nodes.length;
    const totalEdges = edges.length;
    const nodeTypes = nodes.reduce((acc, node) => {
      acc[node.type] = (acc[node.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    const avgConnections = nodes.reduce((sum, node) => sum + node.connections.length, 0) / totalNodes;
    const maxConnections = Math.max(...nodes.map(node => node.connections.length));
    const mostConnectedNode = nodes.find(node => node.connections.length === maxConnections);
    
    return { totalNodes, totalEdges, nodeTypes, avgConnections, mostConnectedNode };
  };

  const stats = getGraphStats();

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
          case '1':
            e.preventDefault();
            handleLayoutChange('force');
            break;
          case '2':
            e.preventDefault();
            handleLayoutChange('hierarchical');
            break;
          case '3':
            e.preventDefault();
            handleLayoutChange('circular');
            break;
          case '0':
            e.preventDefault();
            handleResetView();
            break;
        }
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [handleLayoutChange, handleResetView]);

  return (
    <div>
      <PageHeader
        title="Enterprise Knowledge Graph"
        description="Advanced graph analytics and visualization platform for exploring complex knowledge relationships and semantic networks"
        actions={
          <div className="flex space-x-2">
            <div className="flex space-x-1 bg-muted p-1 rounded-lg">
              <Button 
                variant={viewMode === '2d' ? 'default' : 'outline'}
                onClick={() => setViewMode('2d')}
                size="sm"
              >
                2D View
              </Button>
              <Button 
                variant={viewMode === '3d' ? 'default' : 'outline'}
                onClick={() => setViewMode('3d')}
                size="sm"
              >
                3D View
              </Button>
            </div>
            <div className="h-6 w-px bg-border"></div>
            <Button 
              variant={layoutMode === 'force' ? 'default' : 'outline'}
              onClick={() => handleLayoutChange('force')}
              size="sm"
            >
              Force
            </Button>
            <Button 
              variant={layoutMode === 'hierarchical' ? 'default' : 'outline'}
              onClick={() => handleLayoutChange('hierarchical')}
              size="sm"
            >
              Hierarchical
            </Button>
            <Button 
              variant={layoutMode === 'circular' ? 'default' : 'outline'}
              onClick={() => handleLayoutChange('circular')}
              size="sm"
            >
              Circular
            </Button>
          </div>
        }
      />

      {/* Advanced Analytics Dashboard */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4 mb-6">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Total Nodes</p>
                <p className="text-2xl font-bold">{stats.totalNodes}</p>
              </div>
              <div className="w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center">
                <svg className="w-4 h-4 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6a2 2 0 01-2 2H9a2 2 0 01-2-2V9a2 2 0 012-2h2m7 0V5a2 2 0 00-2-2H9a2 2 0 00-2 2v2m7 0h2m-7 0h2" />
                </svg>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Relationships</p>
                <p className="text-2xl font-bold">{stats.totalEdges}</p>
              </div>
              <div className="w-8 h-8 bg-green-100 dark:bg-green-900 rounded-lg flex items-center justify-center">
                <svg className="w-4 h-4 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Avg Connections</p>
                <p className="text-2xl font-bold">{stats.avgConnections.toFixed(1)}</p>
              </div>
              <div className="w-8 h-8 bg-yellow-100 dark:bg-yellow-900 rounded-lg flex items-center justify-center">
                <svg className="w-4 h-4 text-yellow-600 dark:text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Clusters</p>
                <p className="text-2xl font-bold">{clusters.length}</p>
              </div>
              <div className="w-8 h-8 bg-purple-100 dark:bg-purple-900 rounded-lg flex items-center justify-center">
                <svg className="w-4 h-4 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-muted-foreground">Hub Node</p>
                <p className="text-lg font-bold truncate">{stats.mostConnectedNode?.label}</p>
              </div>
              <div className="w-8 h-8 bg-red-100 dark:bg-red-900 rounded-lg flex items-center justify-center">
                <svg className="w-4 h-4 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Advanced Controls Panel */}
        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <span>Search & Filter</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex space-x-2">
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Search nodes, tags..."
                  className="flex-1"
                />
                <Button onClick={handleSearch} size="sm">
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </Button>
              </div>
              
              <div>
                <label className="text-sm font-medium mb-2 block">Node Type</label>
                <select 
                  value={filterType} 
                  onChange={(e) => setFilterType(e.target.value as any)}
                  className="w-full p-2 border border-input rounded-md bg-background text-sm"
                >
                  <option value="all">All Types</option>
                  <option value="concept">Concepts</option>
                  <option value="entity">Entities</option>
                  <option value="document">Documents</option>
                  <option value="person">People</option>
                </select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <label htmlFor="showClusters" className="text-sm font-medium">Clusters</label>
                  <input
                    type="checkbox"
                    id="showClusters"
                    checked={showClusters}
                    onChange={(e) => setShowClusters(e.target.checked)}
                    className="rounded"
                  />
                </div>

                <div className="flex items-center justify-between">
                  <label htmlFor="showLabels" className="text-sm font-medium">Labels</label>
                  <input
                    type="checkbox"
                    id="showLabels"
                    checked={showLabels}
                    onChange={(e) => setShowLabels(e.target.checked)}
                    className="rounded"
                  />
                </div>
                
                <div className="pt-2 border-t border-border">
                  <div className="text-xs text-muted-foreground space-y-1">
                    <div className="flex justify-between">
                      <span>Layout:</span>
                      <span className="capitalize font-medium">{layoutMode}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>View:</span>
                      <span className="uppercase font-medium">{viewMode}</span>
                    </div>
                  </div>
                </div>
                
                <div className="pt-2 border-t border-border">
                  <div className="text-xs text-muted-foreground">
                    <div className="font-medium mb-1">Shortcuts:</div>
                    <div className="space-y-0.5">
                      <div>Ctrl+1: Force Layout</div>
                      <div>Ctrl+2: Hierarchical</div>
                      <div>Ctrl+3: Circular</div>
                      <div>Ctrl+0: Reset View</div>
                    </div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
                <span>View Controls</span>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Zoom</span>
                  <div className="flex items-center space-x-2">
                    <Button size="sm" variant="outline" onClick={handleZoomOut}>
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
                      </svg>
                    </Button>
                    <span className="text-sm px-3 py-1 bg-muted rounded font-mono min-w-[60px] text-center">
                      {Math.round(zoom * 100)}%
                    </span>
                    <Button size="sm" variant="outline" onClick={handleZoomIn}>
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                      </svg>
                    </Button>
                  </div>
                </div>
                
                <div className="text-xs text-muted-foreground">
                  <div>Pan: ({Math.round(pan.x)}, {Math.round(pan.y)})</div>
                  <div className="mt-1">Mouse wheel to zoom, drag to pan</div>
                </div>
              </div>
              
              <Button size="sm" variant="outline" onClick={handleResetView} className="w-full">
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Reset View
              </Button>
            </CardContent>
          </Card>

          {selectedNodeData && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2">
                  <div 
                    className="w-4 h-4 rounded-full" 
                    style={{ backgroundColor: selectedNodeData.color }}
                  />
                  <span>Node Details</span>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <h4 className="font-semibold text-lg">{selectedNodeData.label}</h4>
                  <p className="text-sm text-muted-foreground capitalize">{selectedNodeData.type}</p>
                </div>
                
                {selectedNodeData.metadata.description && (
                  <p className="text-sm text-muted-foreground">{selectedNodeData.metadata.description}</p>
                )}
                
                <div className="grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <span className="text-muted-foreground">Confidence</span>
                    <div className="font-medium">{(selectedNodeData.metadata.confidence * 100).toFixed(1)}%</div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Connections</span>
                    <div className="font-medium">{selectedNodeData.connections.length}</div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Importance</span>
                    <div className="font-medium">{(selectedNodeData.importance * 10).toFixed(1)}/10</div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Weight</span>
                    <div className="font-medium">{selectedNodeData.metadata.weight}</div>
                  </div>
                </div>

                <div>
                  <span className="text-sm text-muted-foreground">Tags</span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {selectedNodeData.metadata.tags.map(tag => (
                      <span key={tag} className="px-2 py-1 text-xs bg-muted rounded-full">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                <div>
                  <span className="text-sm text-muted-foreground">Source</span>
                  <p className="text-xs mt-1 font-mono bg-muted p-2 rounded">{selectedNodeData.metadata.source}</p>
                </div>

                <div className="space-y-2">
                  <div className="flex space-x-2">
                    <Button 
                      size="sm" 
                      className="flex-1"
                      onClick={() => handleFocusNode(selectedNodeData.id)}
                    >
                      Focus as Root
                    </Button>
                    <Button 
                      size="sm" 
                      variant="outline" 
                      className="flex-1"
                      onClick={() => handleExpandNode(selectedNodeData.id)}
                    >
                      Expand
                    </Button>
                  </div>
                  <div className="flex space-x-2">
                    <Button 
                      size="sm" 
                      variant="outline" 
                      className="flex-1"
                      onClick={handleCollapseView}
                    >
                      Reset View
                    </Button>
                    <Button size="sm" variant="outline" className="flex-1">
                      Export
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Enterprise Graph Visualization */}
        <div className="lg:col-span-3">
          <Card className="h-[700px] overflow-hidden">
            <CardHeader className="pb-2">
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center space-x-2">
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                  </svg>
                  <span>Knowledge Network</span>
                </CardTitle>
                <div className="flex items-center space-x-4 text-sm text-muted-foreground">
                  <div className="flex items-center space-x-2">
                    <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                    <span>Live • {filteredNodes.length} nodes</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="capitalize">{viewMode} Mode</span>
                    <span>•</span>
                    <span className="capitalize">{layoutMode} Layout</span>
                  </div>
                  {focusedNode && (
                    <div className="flex items-center space-x-2">
                      <div className="w-2 h-2 bg-yellow-500 rounded-full"></div>
                      <span>Focused: {nodes.find(n => n.id === focusedNode)?.label}</span>
                    </div>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent className="p-0 h-full">
              <div ref={containerRef} className="h-full w-full">
                {viewMode === '3d' ? (
                  <Suspense fallback={
                    <div className="h-full flex items-center justify-center">
                      <LoadingSpinner />
                      <span className="ml-2">Loading 3D Knowledge Graph...</span>
                    </div>
                  }>
                    <Canvas className="h-full w-full bg-gradient-to-br from-slate-900 to-slate-800">
                      <Scene3D
                        nodes={filteredNodes}
                        edges={filteredEdges}
                        selectedNode={selectedNode}
                        focusedNode={focusedNode}
                        onNodeSelect={handleNodeSelect}
                      />
                    </Canvas>
                  </Suspense>
                ) : (
                  <EnterpriseGraphVisualization
                    nodes={filteredNodes}
                    edges={filteredEdges}
                    clusters={showClusters ? clusters : []}
                    selectedNode={selectedNode}
                    hoveredNode={hoveredNode}
                    onNodeSelect={handleNodeSelect}
                    onNodeHover={handleNodeHover}
                    width={800}
                    height={600}
                    zoom={zoom}
                    pan={pan}
                    onZoomChange={handleZoomChange}
                    onPanChange={handlePanChange}
                  />
                )}
              </div>
            </CardContent>
          </Card>

          {/* Enhanced Legend */}
          <Card className="mt-4">
            <CardHeader>
              <CardTitle>Legend & Node Types</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                <div className="flex items-center space-x-3">
                  <div className="w-6 h-6 bg-blue-500 rounded-full flex items-center justify-center">
                    <div className="w-3 h-3 bg-white rounded-full"></div>
                  </div>
                  <div>
                    <div className="text-sm font-medium">Concepts</div>
                    <div className="text-xs text-muted-foreground">{stats.nodeTypes.concept || 0} nodes</div>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <div className="w-6 h-6 bg-green-500 flex items-center justify-center">
                    <div className="w-3 h-3 bg-white"></div>
                  </div>
                  <div>
                    <div className="text-sm font-medium">Entities</div>
                    <div className="text-xs text-muted-foreground">{stats.nodeTypes.entity || 0} nodes</div>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <div className="w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center">
                    <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M12 2C13.1 2 14 2.9 14 4C14 5.1 13.1 6 12 6C10.9 6 10 5.1 10 4C10 2.9 10.9 2 12 2ZM21 9V7L15 1H5C3.89 1 3 1.89 3 3V21C3 22.1 3.89 23 5 23H19C20.1 23 21 22.1 21 21V9M19 9H14V4H5V21H19V9Z" />
                    </svg>
                  </div>
                  <div>
                    <div className="text-sm font-medium">People</div>
                    <div className="text-xs text-muted-foreground">{stats.nodeTypes.person || 0} nodes</div>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <div className="w-6 h-1 bg-gray-500 rounded"></div>
                  <div>
                    <div className="text-sm font-medium">Relationships</div>
                    <div className="text-xs text-muted-foreground">Curved connections</div>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <div className="w-6 h-6 border-2 border-dashed border-blue-400 rounded-full bg-blue-50 dark:bg-blue-900/20"></div>
                  <div>
                    <div className="text-sm font-medium">Clusters</div>
                    <div className="text-xs text-muted-foreground">Semantic groups</div>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <div className="w-4 h-4 bg-gradient-to-r from-red-500 to-green-500 rounded-full"></div>
                  <div>
                    <div className="text-sm font-medium">Importance</div>
                    <div className="text-xs text-muted-foreground">Color intensity</div>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default KnowledgeGraph;