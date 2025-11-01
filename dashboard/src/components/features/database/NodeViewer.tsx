import React, { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { useDatabaseNodes, useDatabaseNode } from '@/hooks/useDatabase';
import { formatDate, truncateText } from '@/lib/utils';
import type { DatabaseNode } from '@/types/database';

interface NodeViewerProps {
  className?: string;
  selectedType?: string;
  onTypeChange?: (type: string) => void;
}

interface NodeCardProps {
  node: DatabaseNode;
  onClick: (node: DatabaseNode) => void;
  isSelected?: boolean;
}

const NodeCard: React.FC<NodeCardProps> = ({ node, onClick, isSelected }) => {
  const getNodeIcon = (type: string) => {
    switch (type) {
      case 'conversation':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        );
      case 'message':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
          </svg>
        );
      case 'document':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        );
      case 'entity':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
          </svg>
        );
      case 'embedding':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
          </svg>
        );
      default:
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6a2 2 0 01-2 2H9a2 2 0 01-2-2V9a2 2 0 012-2h2m7 0V5a2 2 0 00-2-2H9a2 2 0 00-2 2v2m7 0h2m-7 0h2" />
          </svg>
        );
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'conversation': return 'text-blue-600 bg-blue-50 dark:bg-blue-900/20';
      case 'message': return 'text-green-600 bg-green-50 dark:bg-green-900/20';
      case 'document': return 'text-purple-600 bg-purple-50 dark:bg-purple-900/20';
      case 'entity': return 'text-orange-600 bg-orange-50 dark:bg-orange-900/20';
      case 'embedding': return 'text-pink-600 bg-pink-50 dark:bg-pink-900/20';
      default: return 'text-gray-600 bg-gray-50 dark:bg-gray-900/20';
    }
  };

  const getDisplayTitle = (node: DatabaseNode) => {
    return node.properties?.title || 
           node.properties?.name || 
           node.properties?.content?.substring(0, 50) || 
           `${node.type} ${node.id.substring(0, 8)}`;
  };

  const getDisplayContent = (node: DatabaseNode) => {
    if (node.properties?.content) {
      return truncateText(node.properties.content, 120);
    }
    if (node.properties?.description) {
      return truncateText(node.properties.description, 120);
    }
    return 'No content preview available';
  };

  return (
    <div 
      className={`
        border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md
        ${isSelected ? 'ring-2 ring-primary border-primary' : 'hover:border-primary/50'}
      `}
      onClick={() => onClick(node)}
    >
      <div className="flex items-start space-x-3">
        <div className={`p-2 rounded-lg ${getTypeColor(node.type)}`}>
          {getNodeIcon(node.type)}
        </div>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-2">
            <h4 className="font-medium truncate">{getDisplayTitle(node)}</h4>
            <Badge variant="outline" className="ml-2 capitalize">
              {node.type}
            </Badge>
          </div>
          
          <p className="text-sm text-muted-foreground mb-3">
            {getDisplayContent(node)}
          </p>
          
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>ID: {node.id.substring(0, 12)}...</span>
            <span>{formatDate(node.updated_at)}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

interface NodeDetailsProps {
  nodeId: string;
  onClose: () => void;
}

const NodeDetails: React.FC<NodeDetailsProps> = ({ nodeId, onClose }) => {
  const { data: node, isLoading, error } = useDatabaseNode(nodeId);

  if (isLoading) {
    return (
      <Card className="h-full">
        <CardContent className="flex items-center justify-center h-64">
          <LoadingSpinner />
        </CardContent>
      </Card>
    );
  }

  if (error || !node) {
    return (
      <Card className="h-full border-destructive">
        <CardContent className="flex items-center justify-center h-64">
          <div className="text-center text-destructive">
            <p>Failed to load node details</p>
            <Button variant="outline" onClick={onClose} className="mt-2">
              Close
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="h-full">
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle className="capitalize">{node.type} Details</CardTitle>
          <CardDescription>ID: {node.id}</CardDescription>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </Button>
      </CardHeader>
      
      <CardContent className="space-y-4">
        {/* Basic Information */}
        <div>
          <h4 className="font-medium mb-2">Basic Information</h4>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span className="text-muted-foreground">Type:</span>
              <Badge variant="outline" className="ml-2 capitalize">{node.type}</Badge>
            </div>
            <div>
              <span className="text-muted-foreground">Created:</span>
              <span className="ml-2">{formatDate(node.created_at)}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Updated:</span>
              <span className="ml-2">{formatDate(node.updated_at)}</span>
            </div>
          </div>
        </div>

        {/* Properties */}
        <div>
          <h4 className="font-medium mb-2">Properties</h4>
          <div className="space-y-2">
            {Object.entries(node.properties || {}).map(([key, value]) => (
              <div key={key} className="border rounded p-3">
                <div className="flex items-start justify-between">
                  <span className="font-medium text-sm capitalize">
                    {key.replace(/_/g, ' ')}:
                  </span>
                </div>
                <div className="mt-1 text-sm text-muted-foreground">
                  {typeof value === 'string' ? (
                    value.length > 200 ? (
                      <details>
                        <summary className="cursor-pointer hover:text-foreground">
                          {truncateText(value, 100)} (click to expand)
                        </summary>
                        <div className="mt-2 whitespace-pre-wrap">{value}</div>
                      </details>
                    ) : (
                      <span className="whitespace-pre-wrap">{value}</span>
                    )
                  ) : (
                    <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
                      {JSON.stringify(value, null, 2)}
                    </pre>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Metadata */}
        {node.metadata && Object.keys(node.metadata).length > 0 && (
          <div>
            <h4 className="font-medium mb-2">Metadata</h4>
            <pre className="text-xs bg-muted p-3 rounded overflow-x-auto">
              {JSON.stringify(node.metadata, null, 2)}
            </pre>
          </div>
        )}

        {/* Actions */}
        <div className="flex space-x-2 pt-4 border-t">
          <Button variant="outline" size="sm">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            View Relationships
          </Button>
          <Button variant="outline" size="sm">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            Copy ID
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

export const NodeViewer: React.FC<NodeViewerProps> = ({ 
  className, 
  selectedType = 'all',
  onTypeChange 
}) => {
  const [currentPage, setCurrentPage] = useState(0);
  const [selectedNode, setSelectedNode] = useState<DatabaseNode | null>(null);
  const pageSize = 20;

  const { 
    data: nodesData, 
    isLoading, 
    error 
  } = useDatabaseNodes(
    selectedType === 'all' ? undefined : selectedType, 
    pageSize, 
    currentPage * pageSize
  );

  const nodeTypes = [
    { value: 'all', label: 'All Types' },
    { value: 'conversation', label: 'Conversations' },
    { value: 'message', label: 'Messages' },
    { value: 'document', label: 'Documents' },
    { value: 'entity', label: 'Entities' },
    { value: 'embedding', label: 'Embeddings' },
  ];

  const handleNodeClick = (node: DatabaseNode) => {
    setSelectedNode(node);
  };

  const handleTypeChange = (type: string) => {
    setCurrentPage(0);
    setSelectedNode(null);
    onTypeChange?.(type);
  };

  if (error) {
    return (
      <Card className="border-destructive">
        <CardContent className="pt-6">
          <div className="text-center text-destructive">
            <p>Failed to load database nodes</p>
            <p className="text-sm mt-2">Please check your connection and try again</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Type Filter */}
      <Card>
        <CardHeader>
          <CardTitle>Browse Database Nodes</CardTitle>
          <CardDescription>
            Explore conversations, messages, documents, and other data nodes
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {nodeTypes.map((type) => (
              <Button
                key={type.value}
                variant={selectedType === type.value ? "default" : "outline"}
                size="sm"
                onClick={() => handleTypeChange(type.value)}
              >
                {type.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Main Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Node List */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>
                  {selectedType === 'all' ? 'All Nodes' : `${selectedType}s`.replace(/^./, c => c.toUpperCase())}
                </CardTitle>
                {nodesData?.total && (
                  <Badge variant="secondary">
                    {nodesData.total} total
                  </Badge>
                )}
              </div>
            </CardHeader>
            <CardContent>
              {isLoading ? (
                <div className="flex items-center justify-center h-64">
                  <LoadingSpinner />
                </div>
              ) : nodesData?.nodes?.length ? (
                <div className="space-y-3">
                  {nodesData.nodes.map((node: DatabaseNode) => (
                    <NodeCard
                      key={node.id}
                      node={node}
                      onClick={handleNodeClick}
                      isSelected={selectedNode?.id === node.id}
                    />
                  ))}
                  
                  {/* Pagination */}
                  <div className="flex items-center justify-between pt-4 border-t">
                    <div className="text-sm text-muted-foreground">
                      Showing {currentPage * pageSize + 1} to {Math.min((currentPage + 1) * pageSize, nodesData?.total || 0)} of {nodesData?.total || 0}
                    </div>
                    <div className="flex space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setCurrentPage(p => Math.max(0, p - 1))}
                        disabled={currentPage === 0}
                      >
                        Previous
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setCurrentPage(p => p + 1)}
                        disabled={!nodesData?.has_more}
                      >
                        Next
                      </Button>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-center py-12 text-muted-foreground">
                  <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                  </svg>
                  <p>No nodes found</p>
                  <p className="text-sm">Try selecting a different type or check your filters</p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Node Details */}
        <div className="lg:col-span-1">
          {selectedNode ? (
            <NodeDetails
              nodeId={selectedNode.id}
              onClose={() => setSelectedNode(null)}
            />
          ) : (
            <Card>
              <CardContent className="flex items-center justify-center h-64">
                <div className="text-center text-muted-foreground">
                  <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                  <p>Select a node to view details</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};