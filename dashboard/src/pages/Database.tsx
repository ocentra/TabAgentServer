import React, { useState } from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { DatabaseExplorer } from '@/components/features/database/DatabaseExplorer';
import { NodeViewer } from '@/components/features/database/NodeViewer';
import { SearchInterface } from '@/components/features/database/SearchInterface';
import { DataExport } from '@/components/features/database/DataExport';
import { KnowledgeGraphVisualization } from '@/components/features/database/KnowledgeGraphVisualization';
import { DataManagement } from '@/components/features/database/DataManagement';
import type { DatabaseSearchResult, KnowledgeGraphNode, KnowledgeGraphEdge } from '@/types/database';

const Database: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'overview' | 'browse' | 'search' | 'graph' | 'manage' | 'export'>('overview');
  const [selectedNodeType, setSelectedNodeType] = useState('all');
  // State for selected items (for future use in detailed views)
  const [, setSelectedResult] = useState<DatabaseSearchResult | null>(null);
  const [, setSelectedGraphNode] = useState<KnowledgeGraphNode | null>(null);
  const [, setSelectedGraphEdge] = useState<KnowledgeGraphEdge | null>(null);

  const tabs = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'browse', label: 'Browse Data', icon: '🗂️' },
    { id: 'search', label: 'Search', icon: '🔍' },
    { id: 'graph', label: 'Knowledge Graph', icon: '🕸️' },
    { id: 'manage', label: 'Data Management', icon: '⚙️' },
    { id: 'export', label: 'Export', icon: '📤' },
  ];

  const handleSearchResultSelect = (result: DatabaseSearchResult) => {
    setSelectedResult(result);
    // Could navigate to detailed view or show in modal
    console.log('Selected search result:', result);
  };

  const handleGraphNodeSelect = (node: KnowledgeGraphNode) => {
    setSelectedGraphNode(node);
    console.log('Selected graph node:', node);
  };

  const handleGraphEdgeSelect = (edge: KnowledgeGraphEdge) => {
    setSelectedGraphEdge(edge);
    console.log('Selected graph edge:', edge);
  };

  return (
    <div>
      <PageHeader
        title="Database Explorer"
        description="Browse and analyze stored conversations, documents, and knowledge graph data"
        actions={
          <div className="flex space-x-2">
            <Button 
              variant="outline" 
              onClick={() => setActiveTab('export')}
            >
              Export Data
            </Button>
            <Button 
              onClick={() => setActiveTab('search')}
            >
              Search Database
            </Button>
          </div>
        }
      />

      {/* Tab Navigation */}
      <div className="mb-6">
        <div className="border-b border-border">
          <nav className="-mb-px flex space-x-8">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`
                  py-2 px-1 border-b-2 font-medium text-sm transition-colors
                  ${activeTab === tab.id
                    ? 'border-primary text-primary'
                    : 'border-transparent text-muted-foreground hover:text-foreground hover:border-gray-300'
                  }
                `}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      <div className="space-y-6">
        {activeTab === 'overview' && <DatabaseExplorer />}
        
        {activeTab === 'browse' && (
          <NodeViewer 
            selectedType={selectedNodeType}
            onTypeChange={setSelectedNodeType}
          />
        )}
        
        {activeTab === 'search' && (
          <SearchInterface onResultSelect={handleSearchResultSelect} />
        )}
        
        {activeTab === 'graph' && (
          <KnowledgeGraphVisualization 
            onNodeSelect={handleGraphNodeSelect}
            onEdgeSelect={handleGraphEdgeSelect}
          />
        )}
        
        {activeTab === 'manage' && (
          <DataManagement />
        )}
        
        {activeTab === 'export' && (
          <DataExport onClose={() => setActiveTab('overview')} />
        )}
      </div>
    </div>
  );
};

export default Database;