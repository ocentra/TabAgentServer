import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { useDatabaseStats, useDatabaseSchema, useDatabasePerformance } from '@/hooks/useDatabase';
import { formatBytes, formatNumber, formatDuration } from '@/lib/utils';

interface DatabaseExplorerProps {
  className?: string;
}

export const DatabaseExplorer: React.FC<DatabaseExplorerProps> = ({ className }) => {
  const { data: stats, isLoading: statsLoading, error: statsError } = useDatabaseStats();
  const { data: schema, isLoading: schemaLoading } = useDatabaseSchema();
  const { data: performance, isLoading: performanceLoading } = useDatabasePerformance();

  if (statsLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (statsError) {
    return (
      <Card className="border-destructive">
        <CardContent className="pt-6">
          <div className="text-center text-destructive">
            <p>Failed to load database information</p>
            <p className="text-sm mt-2">Please check your connection and try again</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Database Overview Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Nodes</CardTitle>
            <svg className="w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14-7l2 2m0 0l2 2m-2-2v6a2 2 0 01-2 2H9a2 2 0 01-2-2V9a2 2 0 012-2h2m7 0V5a2 2 0 00-2-2H9a2 2 0 00-2 2v2m7 0h2m-7 0h2" />
            </svg>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(stats?.nodes || 0)}</div>
            <CardDescription>Data nodes in graph</CardDescription>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Relationships</CardTitle>
            <svg className="w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatNumber(stats?.edges || 0)}</div>
            <CardDescription>Knowledge connections</CardDescription>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Storage Size</CardTitle>
            <svg className="w-4 h-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
            </svg>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatBytes(stats?.size_bytes || 0)}</div>
            <CardDescription>Total database size</CardDescription>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Health Status</CardTitle>
            <div className={`w-3 h-3 rounded-full ${
              stats?.health?.status === 'healthy' ? 'bg-green-500' :
              stats?.health?.status === 'degraded' ? 'bg-yellow-500' : 'bg-red-500'
            }`} />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold capitalize">
              {stats?.health?.status || 'Unknown'}
            </div>
            <CardDescription>
              {stats?.health?.disk_usage_percent ? 
                `${stats.health.disk_usage_percent}% disk usage` : 
                'Database status'
              }
            </CardDescription>
          </CardContent>
        </Card>
      </div>

      {/* Collections Breakdown */}
      <Card>
        <CardHeader>
          <CardTitle>Collections Overview</CardTitle>
          <CardDescription>Distribution of data across different collections</CardDescription>
        </CardHeader>
        <CardContent>
          {stats?.collections ? (
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {Object.entries(stats.collections).map(([collection, count]) => (
                <div key={collection} className="flex items-center justify-between p-3 border rounded-lg">
                  <div>
                    <p className="font-medium capitalize">{collection.replace('_', ' ')}</p>
                    <p className="text-sm text-muted-foreground">{formatNumber(count)} items</p>
                  </div>
                  <Badge variant="secondary">{formatNumber(count)}</Badge>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              No collection data available
            </div>
          )}
        </CardContent>
      </Card>

      {/* Database Schema */}
      <Card>
        <CardHeader>
          <CardTitle>Database Schema</CardTitle>
          <CardDescription>Node and edge types with their properties</CardDescription>
        </CardHeader>
        <CardContent>
          {schemaLoading ? (
            <div className="flex items-center justify-center py-8">
              <LoadingSpinner />
            </div>
          ) : schema ? (
            <div className="space-y-6">
              {/* Node Types */}
              <div>
                <h4 className="font-medium mb-3">Node Types</h4>
                <div className="grid gap-3">
                  {schema.node_types?.map((nodeType) => (
                    <div key={nodeType.type} className="border rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <h5 className="font-medium capitalize">{nodeType.type}</h5>
                        <Badge variant="outline">{formatNumber(nodeType.count)} nodes</Badge>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        {nodeType.properties?.map((prop) => (
                          <Badge 
                            key={prop.name} 
                            variant={prop.indexed ? "default" : "secondary"}
                            className="text-xs"
                          >
                            {prop.name}: {prop.type}
                            {prop.indexed && ' 🔍'}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Edge Types */}
              <div>
                <h4 className="font-medium mb-3">Relationship Types</h4>
                <div className="grid gap-3">
                  {schema.edge_types?.map((edgeType) => (
                    <div key={edgeType.type} className="border rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <h5 className="font-medium uppercase">{edgeType.type}</h5>
                        <Badge variant="outline">{formatNumber(edgeType.count)} relationships</Badge>
                      </div>
                      <div className="text-sm text-muted-foreground">
                        <span className="font-medium">From:</span> {edgeType.source_types?.join(', ')}
                        <br />
                        <span className="font-medium">To:</span> {edgeType.target_types?.join(', ')}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              Schema information not available
            </div>
          )}
        </CardContent>
      </Card>

      {/* Performance Metrics */}
      <Card>
        <CardHeader>
          <CardTitle>Performance Metrics</CardTitle>
          <CardDescription>Database query performance and statistics</CardDescription>
        </CardHeader>
        <CardContent>
          {performanceLoading ? (
            <div className="flex items-center justify-center py-8">
              <LoadingSpinner />
            </div>
          ) : performance ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-blue-600">
                  {formatNumber(performance.queries_per_second || 0)}
                </div>
                <div className="text-sm text-muted-foreground">Queries/sec</div>
              </div>
              
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-green-600">
                  {formatDuration(performance.average_query_time || 0)}
                </div>
                <div className="text-sm text-muted-foreground">Avg Query Time</div>
              </div>
              
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-purple-600">
                  {Math.round((performance.cache_stats?.hit_rate || 0) * 100)}%
                </div>
                <div className="text-sm text-muted-foreground">Cache Hit Rate</div>
              </div>
              
              <div className="text-center p-4 border rounded-lg">
                <div className="text-2xl font-bold text-orange-600">
                  {formatBytes(stats?.health?.memory_usage_mb ? stats.health.memory_usage_mb * 1024 * 1024 : 0)}
                </div>
                <div className="text-sm text-muted-foreground">Memory Usage</div>
              </div>
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              Performance metrics not available
            </div>
          )}
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle>Database Management</CardTitle>
          <CardDescription>Quick actions for database maintenance and analysis</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-3">
            <Button variant="outline">
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
              </svg>
              Export Data
            </Button>
            
            <Button variant="outline">
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              Create Backup
            </Button>
            
            <Button variant="outline">
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
              </svg>
              View Analytics
            </Button>
            
            <Button variant="outline">
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh Data
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};