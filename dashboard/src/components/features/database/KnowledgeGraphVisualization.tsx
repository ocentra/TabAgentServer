import React, { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useKnowledgeGraph } from '@/hooks/useDatabase';
import type { KnowledgeGraphNode, KnowledgeGraphEdge } from '@/types/database';

interface KnowledgeGraphVisualizationProps {
  className?: string;
  filters?: any;
  onNodeSelect?: (node: KnowledgeGraphNode) => void;
  onEdgeSelect?: (edge: KnowledgeGraphEdge) => void;
}

interface GraphFilters {
  nodeTypes: string[];
  edgeTypes: string[];
  minConnections: number;
  searchTerm: string;
}

interface SimulationNode extends KnowledgeGraphNode {
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  vx?: number;
  vy?: number;
}

interface SimulationLink {
  id: string;
  source: SimulationNode | string;
  target: SimulationNode | string;
  type: string;
  weight?: number;
  properties?: Record<string, any>;
}

export const KnowledgeGraphVisualization: React.FC<KnowledgeGraphVisualizationProps> = ({
  className,
  filters: externalFilters,
  onNodeSelect,
  onEdgeSelect,
}) => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const [selectedNode, setSelectedNode] = useState<KnowledgeGraphNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<KnowledgeGraphEdge | null>(null);
  const [filters, setFilters] = useState<GraphFilters>({
    nodeTypes: [],
    edgeTypes: [],
    minConnections: 0,
    searchTerm: '',
  });
  const [zoomLevel, setZoomLevel] = useState(1);

  const { data: graphData, isLoading, error } = useKnowledgeGraph(externalFilters);

  // Color schemes for different node types
  const nodeColors = {
    conversation: '#3b82f6',
    message: '#10b981',
    document: '#8b5cf6',
    entity: '#f59e0b',
    embedding: '#ec4899',
    default: '#6b7280',
  };

  const edgeColors = {
    CONTAINS: '#94a3b8',
    REFERENCES: '#64748b',
    RELATES_TO: '#475569',
    SIMILAR_TO: '#334155',
    default: '#94a3b8',
  };

  // Filter graph data based on current filters
  const filteredGraphData = React.useMemo(() => {
    if (!graphData) return null;

    let nodes = [...graphData.nodes];
    let edges = [...graphData.edges];

    // Filter by node types
    if (filters.nodeTypes.length > 0) {
      nodes = nodes.filter(node => filters.nodeTypes.includes(node.type));
      const nodeIds = new Set(nodes.map(n => n.id));
      edges = edges.filter(edge => 
        nodeIds.has(edge.source as string) && nodeIds.has(edge.target as string)
      );
    }

    // Filter by edge types
    if (filters.edgeTypes.length > 0) {
      edges = edges.filter(edge => filters.edgeTypes.includes(edge.type));
    }

    // Filter by search term
    if (filters.searchTerm) {
      const searchLower = filters.searchTerm.toLowerCase();
      nodes = nodes.filter(node => 
        node.label.toLowerCase().includes(searchLower) ||
        node.type.toLowerCase().includes(searchLower) ||
        Object.values(node.properties || {}).some(value => 
          String(value).toLowerCase().includes(searchLower)
        )
      );
      const nodeIds = new Set(nodes.map(n => n.id));
      edges = edges.filter(edge => 
        nodeIds.has(edge.source as string) && nodeIds.has(edge.target as string)
      );
    }

    // Filter by minimum connections
    if (filters.minConnections > 0) {
      const connectionCounts = new Map<string, number>();
      edges.forEach(edge => {
        connectionCounts.set(edge.source as string, (connectionCounts.get(edge.source as string) || 0) + 1);
        connectionCounts.set(edge.target as string, (connectionCounts.get(edge.target as string) || 0) + 1);
      });
      
      nodes = nodes.filter(node => 
        (connectionCounts.get(node.id) || 0) >= filters.minConnections
      );
      const nodeIds = new Set(nodes.map(n => n.id));
      edges = edges.filter(edge => 
        nodeIds.has(edge.source as string) && nodeIds.has(edge.target as string)
      );
    }

    return { nodes, edges, metadata: graphData.metadata };
  }, [graphData, filters]);

  // Handle container resize
  useEffect(() => {
    const handleResize = () => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        setDimensions({
          width: rect.width,
          height: Math.max(400, rect.height),
        });
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // D3 visualization
  useEffect(() => {
    if (!filteredGraphData || !svgRef.current) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const { width, height } = dimensions;
    
    // Create zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        container.attr('transform', event.transform);
        setZoomLevel(event.transform.k);
      });

    svg.call(zoom);

    // Create container group
    const container = svg.append('g');

    // Create simulation
    const simulation = d3.forceSimulation<SimulationNode>(filteredGraphData.nodes as SimulationNode[])
      .force('link', d3.forceLink<SimulationNode, SimulationLink>(filteredGraphData.edges as SimulationLink[])
        .id((d: any) => d.id)
        .distance(100)
        .strength(0.1)
      )
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30));

    // Create links
    const links = container.append('g')
      .selectAll('line')
      .data(filteredGraphData.edges)
      .enter()
      .append('line')
      .attr('stroke', (d: any) => edgeColors[d.type as keyof typeof edgeColors] || edgeColors.default)
      .attr('stroke-width', (d: any) => Math.sqrt(d.weight || 1) * 2)
      .attr('stroke-opacity', 0.6)
      .style('cursor', 'pointer')
      .on('click', (_, d) => {
        setSelectedEdge(d as KnowledgeGraphEdge);
        onEdgeSelect?.(d as KnowledgeGraphEdge);
      })
      .on('mouseover', function(event, d) {
        d3.select(this).attr('stroke-opacity', 1);
        
        // Show tooltip
        const tooltip = d3.select('body').append('div')
          .attr('class', 'tooltip')
          .style('position', 'absolute')
          .style('background', 'rgba(0, 0, 0, 0.8)')
          .style('color', 'white')
          .style('padding', '8px')
          .style('border-radius', '4px')
          .style('font-size', '12px')
          .style('pointer-events', 'none')
          .style('z-index', '1000')
          .html(`
            <strong>${(d as any).type}</strong><br/>
            Weight: ${(d as any).weight || 1}
          `);
        
        tooltip.style('left', (event.pageX + 10) + 'px')
          .style('top', (event.pageY - 10) + 'px');
      })
      .on('mouseout', function() {
        d3.select(this).attr('stroke-opacity', 0.6);
        d3.selectAll('.tooltip').remove();
      });

    // Create nodes
    const nodes = container.append('g')
      .selectAll('circle')
      .data(filteredGraphData.nodes)
      .enter()
      .append('circle')
      .attr('r', (d: any) => Math.sqrt((d.size || 10)) + 5)
      .attr('fill', (d: any) => nodeColors[d.type as keyof typeof nodeColors] || nodeColors.default)
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .call(d3.drag<SVGCircleElement, SimulationNode>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        })
      )
      .on('click', (_, d) => {
        setSelectedNode(d as KnowledgeGraphNode);
        onNodeSelect?.(d as KnowledgeGraphNode);
      })
      .on('mouseover', function(event, d) {
        d3.select(this).attr('stroke-width', 4);
        
        // Show tooltip
        const tooltip = d3.select('body').append('div')
          .attr('class', 'tooltip')
          .style('position', 'absolute')
          .style('background', 'rgba(0, 0, 0, 0.8)')
          .style('color', 'white')
          .style('padding', '8px')
          .style('border-radius', '4px')
          .style('font-size', '12px')
          .style('pointer-events', 'none')
          .style('z-index', '1000')
          .html(`
            <strong>${(d as any).label}</strong><br/>
            Type: ${(d as any).type}<br/>
            ID: ${(d as any).id.substring(0, 8)}...
          `);
        
        tooltip.style('left', (event.pageX + 10) + 'px')
          .style('top', (event.pageY - 10) + 'px');
      })
      .on('mouseout', function() {
        d3.select(this).attr('stroke-width', 2);
        d3.selectAll('.tooltip').remove();
      });

    // Create labels
    const labels = container.append('g')
      .selectAll('text')
      .data(filteredGraphData.nodes)
      .enter()
      .append('text')
      .text((d: any) => d.label.length > 15 ? d.label.substring(0, 15) + '...' : d.label)
      .attr('font-size', '10px')
      .attr('font-family', 'Arial, sans-serif')
      .attr('fill', '#333')
      .attr('text-anchor', 'middle')
      .attr('dy', '.35em')
      .style('pointer-events', 'none');

    // Update positions on simulation tick
    simulation.on('tick', () => {
      links
        .attr('x1', (d: any) => (d.source as SimulationNode).x!)
        .attr('y1', (d: any) => (d.source as SimulationNode).y!)
        .attr('x2', (d: any) => (d.target as SimulationNode).x!)
        .attr('y2', (d: any) => (d.target as SimulationNode).y!);

      nodes
        .attr('cx', (d: any) => d.x!)
        .attr('cy', (d: any) => d.y!);

      labels
        .attr('x', (d: any) => d.x!)
        .attr('y', (d: any) => d.y! + 25);
    });

    return () => {
      simulation.stop();
      d3.selectAll('.tooltip').remove();
    };
  }, [filteredGraphData, dimensions, onNodeSelect, onEdgeSelect]);

  const handleFilterChange = (key: keyof GraphFilters, value: any) => {
    setFilters(prev => ({ ...prev, [key]: value }));
  };

  const resetFilters = () => {
    setFilters({
      nodeTypes: [],
      edgeTypes: [],
      minConnections: 0,
      searchTerm: '',
    });
  };

  const centerGraph = () => {
    if (svgRef.current) {
      const svg = d3.select(svgRef.current);
      // Center the graph view
      
      svg.transition()
        .duration(750)
        .call(
          d3.zoom<SVGSVGElement, unknown>().transform,
          d3.zoomIdentity.translate(0, 0).scale(1)
        );
    }
  };

  if (isLoading) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-96">
          <LoadingSpinner size="lg" />
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={`border-destructive ${className}`}>
        <CardContent className="pt-6">
          <div className="text-center text-destructive">
            <p>Failed to load knowledge graph</p>
            <p className="text-sm mt-2">Please check your connection and try again</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!graphData || graphData.nodes.length === 0) {
    return (
      <Card className={className}>
        <CardContent className="flex items-center justify-center h-96">
          <div className="text-center text-muted-foreground">
            <svg className="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            <p>No graph data available</p>
            <p className="text-sm">The knowledge graph is empty or no data matches your filters</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Controls */}
      <Card>
        <CardHeader>
          <CardTitle>Knowledge Graph Visualization</CardTitle>
          <CardDescription>
            Interactive visualization of relationships between data nodes
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Search and Filters */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Search Nodes</label>
              <Input
                placeholder="Search by label or content..."
                value={filters.searchTerm}
                onChange={(e) => handleFilterChange('searchTerm', e.target.value)}
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Node Types</label>
              <Select
                value={filters.nodeTypes[0] || 'all'}
                onChange={(value) => 
                  handleFilterChange('nodeTypes', value === 'all' ? [] : [value])
                }
              >
                <option value="all">All Types</option>
                {Object.keys(graphData.metadata.node_types).map(type => (
                  <option key={type} value={type}>{type}</option>
                ))}
              </Select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Min Connections</label>
              <Input
                type="number"
                min="0"
                value={filters.minConnections}
                onChange={(e) => handleFilterChange('minConnections', parseInt(e.target.value) || 0)}
              />
            </div>
            
            <div className="flex items-end space-x-2">
              <Button variant="outline" onClick={resetFilters} size="sm">
                Reset Filters
              </Button>
              <Button variant="outline" onClick={centerGraph} size="sm">
                Center Graph
              </Button>
            </div>
          </div>

          {/* Graph Stats */}
          <div className="flex items-center space-x-6 text-sm text-muted-foreground">
            <span>
              Nodes: <Badge variant="secondary">{filteredGraphData?.nodes.length || 0}</Badge>
            </span>
            <span>
              Edges: <Badge variant="secondary">{filteredGraphData?.edges.length || 0}</Badge>
            </span>
            <span>
              Zoom: <Badge variant="secondary">{Math.round(zoomLevel * 100)}%</Badge>
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Graph Visualization */}
      <Card>
        <CardContent className="p-0">
          <div ref={containerRef} className="w-full h-96 relative">
            <svg
              ref={svgRef}
              width={dimensions.width}
              height={dimensions.height}
              className="border-b"
            />
          </div>
        </CardContent>
      </Card>

      {/* Legend */}
      <Card>
        <CardHeader>
          <CardTitle>Legend</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-6">
            {/* Node Types */}
            <div>
              <h4 className="font-medium mb-3">Node Types</h4>
              <div className="space-y-2">
                {Object.entries(nodeColors).filter(([key]) => key !== 'default').map(([type, color]) => (
                  <div key={type} className="flex items-center space-x-2">
                    <div 
                      className="w-4 h-4 rounded-full border-2 border-white"
                      style={{ backgroundColor: color }}
                    />
                    <span className="text-sm capitalize">{type}</span>
                    <Badge variant="outline" className="text-xs">
                      {graphData.metadata.node_types[type] || 0}
                    </Badge>
                  </div>
                ))}
              </div>
            </div>

            {/* Edge Types */}
            <div>
              <h4 className="font-medium mb-3">Relationship Types</h4>
              <div className="space-y-2">
                {Object.entries(edgeColors).filter(([key]) => key !== 'default').map(([type, color]) => (
                  <div key={type} className="flex items-center space-x-2">
                    <div 
                      className="w-6 h-0.5"
                      style={{ backgroundColor: color }}
                    />
                    <span className="text-sm">{type.replace(/_/g, ' ')}</span>
                    <Badge variant="outline" className="text-xs">
                      {graphData.metadata.edge_types[type] || 0}
                    </Badge>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Selected Node/Edge Details */}
      {(selectedNode || selectedEdge) && (
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>
                {selectedNode ? 'Node Details' : 'Relationship Details'}
              </CardTitle>
              <Button 
                variant="ghost" 
                size="sm" 
                onClick={() => {
                  setSelectedNode(null);
                  setSelectedEdge(null);
                }}
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {selectedNode && (
              <div className="space-y-2">
                <div><strong>Label:</strong> {selectedNode.label}</div>
                <div><strong>Type:</strong> <Badge variant="outline">{selectedNode.type}</Badge></div>
                <div><strong>ID:</strong> <code className="text-xs">{selectedNode.id}</code></div>
                {Object.keys(selectedNode.properties || {}).length > 0 && (
                  <div>
                    <strong>Properties:</strong>
                    <pre className="text-xs bg-muted p-2 rounded mt-1 overflow-x-auto">
                      {JSON.stringify(selectedNode.properties, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            )}
            {selectedEdge && (
              <div className="space-y-2">
                <div><strong>Type:</strong> <Badge variant="outline">{selectedEdge.type}</Badge></div>
                <div><strong>Source:</strong> <code className="text-xs">{selectedEdge.source}</code></div>
                <div><strong>Target:</strong> <code className="text-xs">{selectedEdge.target}</code></div>
                {selectedEdge.weight && <div><strong>Weight:</strong> {selectedEdge.weight}</div>}
                {selectedEdge.properties && Object.keys(selectedEdge.properties).length > 0 && (
                  <div>
                    <strong>Properties:</strong>
                    <pre className="text-xs bg-muted p-2 rounded mt-1 overflow-x-auto">
                      {JSON.stringify(selectedEdge.properties, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
};