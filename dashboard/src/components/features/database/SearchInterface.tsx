import React, { useState, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Badge } from '@/components/ui/Badge';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Select } from '@/components/ui/Select';
import { useSearchDatabase } from '@/hooks/useDatabase';
import { formatDate, truncateText, debounce } from '@/lib/utils';
import type { DatabaseQuery, DatabaseSearchResult } from '@/types/database';

interface SearchInterfaceProps {
  className?: string;
  onResultSelect?: (result: DatabaseSearchResult) => void;
}

interface SearchFiltersProps {
  filters: DatabaseQuery['filters'];
  onFiltersChange: (filters: DatabaseQuery['filters']) => void;
}

const SearchFilters: React.FC<SearchFiltersProps> = ({ filters, onFiltersChange }) => {
  const updateFilter = (key: string, value: any) => {
    onFiltersChange({
      ...filters,
      [key]: value,
    });
  };

  const updateDateRange = (field: 'start' | 'end', value: string) => {
    onFiltersChange({
      ...filters,
      date_range: {
        start: filters?.date_range?.start || '',
        end: filters?.date_range?.end || '',
        [field]: value,
      },
    });
  };

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* Type Filter */}
        <div>
          <label className="block text-sm font-medium mb-2">Content Type</label>
          <Select
            value={filters?.type?.[0] || 'all'}
            onChange={(value) => 
              updateFilter('type', value === 'all' ? undefined : [value])
            }
          >
            <option value="all">All Types</option>
            <option value="conversation">Conversations</option>
            <option value="message">Messages</option>
            <option value="document">Documents</option>
            <option value="entity">Entities</option>
            <option value="embedding">Embeddings</option>
          </Select>
        </div>

        {/* Date Range */}
        <div>
          <label className="block text-sm font-medium mb-2">Date Range</label>
          <div className="flex space-x-2">
            <Input
              type="date"
              placeholder="Start date"
              value={filters?.date_range?.start?.split('T')[0] || ''}
              onChange={(e) => updateDateRange('start', e.target.value ? `${e.target.value}T00:00:00Z` : '')}
              className="text-sm"
            />
            <Input
              type="date"
              placeholder="End date"
              value={filters?.date_range?.end?.split('T')[0] || ''}
              onChange={(e) => updateDateRange('end', e.target.value ? `${e.target.value}T23:59:59Z` : '')}
              className="text-sm"
            />
          </div>
        </div>

        {/* Quick Date Filters */}
        <div>
          <label className="block text-sm font-medium mb-2">Quick Filters</label>
          <div className="flex flex-wrap gap-2">
            {[
              { label: 'Today', days: 1 },
              { label: 'Week', days: 7 },
              { label: 'Month', days: 30 },
              { label: 'All', days: null },
            ].map((preset) => (
              <Button
                key={preset.label}
                variant="outline"
                size="sm"
                onClick={() => {
                  if (preset.days === null) {
                    updateFilter('date_range', undefined);
                  } else {
                    const end = new Date().toISOString();
                    const start = new Date(Date.now() - preset.days * 24 * 60 * 60 * 1000).toISOString();
                    updateFilter('date_range', { start, end });
                  }
                }}
              >
                {preset.label}
              </Button>
            ))}
          </div>
        </div>
      </div>

      {/* Active Filters Display */}
      {(filters?.type || filters?.date_range) && (
        <div className="flex flex-wrap gap-2 pt-2 border-t">
          <span className="text-sm text-muted-foreground">Active filters:</span>
          {filters?.type && (
            <Badge variant="secondary" className="capitalize">
              Type: {filters.type[0]}
              <button
                onClick={() => updateFilter('type', undefined)}
                className="ml-2 hover:text-destructive"
              >
                ×
              </button>
            </Badge>
          )}
          {filters?.date_range && (
            <Badge variant="secondary">
              Date: {filters.date_range.start?.split('T')[0]} to {filters.date_range.end?.split('T')[0]}
              <button
                onClick={() => updateFilter('date_range', undefined)}
                className="ml-2 hover:text-destructive"
              >
                ×
              </button>
            </Badge>
          )}
        </div>
      )}
    </div>
  );
};

interface SearchResultCardProps {
  result: DatabaseSearchResult;
  onClick: (result: DatabaseSearchResult) => void;
}

const SearchResultCard: React.FC<SearchResultCardProps> = ({ result, onClick }) => {
  const getTypeColor = (type: string) => {
    switch (type) {
      case 'conversation': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200';
      case 'message': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'document': return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200';
      case 'entity': return 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200';
      case 'embedding': return 'bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200';
      default: return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200';
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600';
    if (score >= 0.6) return 'text-yellow-600';
    return 'text-red-600';
  };

  return (
    <div 
      className="border rounded-lg p-4 cursor-pointer transition-all hover:shadow-md hover:border-primary/50"
      onClick={() => onClick(result)}
    >
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center space-x-2">
          <Badge className={getTypeColor(result.type)} variant="secondary">
            {result.type}
          </Badge>
          <span className={`text-sm font-medium ${getScoreColor(result.score)}`}>
            {Math.round(result.score * 100)}% match
          </span>
        </div>
        <span className="text-xs text-muted-foreground">
          {formatDate(result.updated_at)}
        </span>
      </div>

      <h4 className="font-medium mb-2 line-clamp-2">
        {result.title || `${result.type} ${result.id.substring(0, 8)}`}
      </h4>

      <p className="text-sm text-muted-foreground mb-3 line-clamp-3">
        {truncateText(result.content, 150)}
      </p>

      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>ID: {result.id.substring(0, 12)}...</span>
        {result.metadata?.size && (
          <span>{Math.round(result.metadata.size / 1024)} KB</span>
        )}
      </div>
    </div>
  );
};

export const SearchInterface: React.FC<SearchInterfaceProps> = ({ 
  className, 
  onResultSelect 
}) => {
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<DatabaseQuery['filters']>({});
  const [sortBy, setSortBy] = useState<{ field: string; order: 'asc' | 'desc' }>({
    field: 'score',
    order: 'desc',
  });
  const [currentPage, setCurrentPage] = useState(0);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const searchMutation = useSearchDatabase();

  const performSearch = useCallback(
    debounce((searchQuery: string, searchFilters: DatabaseQuery['filters']) => {
      if (!searchQuery.trim()) return;

      const searchParams: DatabaseQuery = {
        query: searchQuery,
        filters: searchFilters,
        sort: sortBy,
        limit: 20,
        offset: currentPage * 20,
      };

      searchMutation.mutate(searchParams);
    }, 300),
    [sortBy, currentPage]
  );

  const handleSearch = () => {
    setCurrentPage(0);
    performSearch(query, filters);
  };

  const handleQueryChange = (newQuery: string) => {
    setQuery(newQuery);
    if (newQuery.trim()) {
      performSearch(newQuery, filters);
    }
  };

  const handleFiltersChange = (newFilters: DatabaseQuery['filters']) => {
    setFilters(newFilters);
    setCurrentPage(0);
    if (query.trim()) {
      performSearch(query, newFilters);
    }
  };

  const handleResultClick = (result: DatabaseSearchResult) => {
    onResultSelect?.(result);
  };

  const clearSearch = () => {
    setQuery('');
    setFilters({});
    setCurrentPage(0);
    searchMutation.reset();
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Search Input */}
      <Card>
        <CardHeader>
          <CardTitle>Search Database</CardTitle>
          <CardDescription>
            Search through conversations, documents, and knowledge graph data
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Main Search */}
          <div className="flex space-x-2">
            <Input
              placeholder="Search conversations, documents, entities..."
              value={query}
              onChange={(e) => handleQueryChange(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              className="flex-1"
            />
            <Button onClick={handleSearch} disabled={!query.trim()}>
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              Search
            </Button>
            {(query || Object.keys(filters || {}).length > 0) && (
              <Button variant="outline" onClick={clearSearch}>
                Clear
              </Button>
            )}
          </div>

          {/* Advanced Filters Toggle */}
          <div className="flex items-center justify-between">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowAdvanced(!showAdvanced)}
            >
              <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4" />
              </svg>
              Advanced Filters
              <svg className={`w-4 h-4 ml-2 transition-transform ${showAdvanced ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </Button>

            {/* Sort Options */}
            <div className="flex items-center space-x-2">
              <span className="text-sm text-muted-foreground">Sort by:</span>
              <Select
                value={`${sortBy.field}-${sortBy.order}`}
                onChange={(value) => {
                  const [field, order] = value.split('-');
                  setSortBy({ field, order: order as 'asc' | 'desc' });
                }}
              >
                <option value="score-desc">Relevance</option>
                <option value="updated_at-desc">Newest First</option>
                <option value="updated_at-asc">Oldest First</option>
                <option value="created_at-desc">Recently Created</option>
              </Select>
            </div>
          </div>

          {/* Advanced Filters */}
          {showAdvanced && (
            <div className="border-t pt-4">
              <SearchFilters filters={filters} onFiltersChange={handleFiltersChange} />
            </div>
          )}
        </CardContent>
      </Card>

      {/* Search Results */}
      {searchMutation.data && (
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle>Search Results</CardTitle>
              <div className="flex items-center space-x-4 text-sm text-muted-foreground">
                <span>
                  {searchMutation.data.total} results in {searchMutation.data.query_time_ms}ms
                </span>
                {searchMutation.data.facets && (
                  <Button variant="ghost" size="sm">
                    View Facets
                  </Button>
                )}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {searchMutation.isPending ? (
              <div className="flex items-center justify-center h-32">
                <LoadingSpinner />
              </div>
            ) : searchMutation.data.results.length > 0 ? (
              <div className="space-y-4">
                {searchMutation.data.results.map((result) => (
                  <SearchResultCard
                    key={result.id}
                    result={result}
                    onClick={handleResultClick}
                  />
                ))}

                {/* Pagination */}
                <div className="flex items-center justify-between pt-4 border-t">
                  <div className="text-sm text-muted-foreground">
                    Showing {currentPage * 20 + 1} to {Math.min((currentPage + 1) * 20, searchMutation.data.total)} of {searchMutation.data.total}
                  </div>
                  <div className="flex space-x-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setCurrentPage(p => Math.max(0, p - 1));
                        performSearch(query, filters);
                      }}
                      disabled={currentPage === 0}
                    >
                      Previous
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setCurrentPage(p => p + 1);
                        performSearch(query, filters);
                      }}
                      disabled={searchMutation.data.results.length < 20}
                    >
                      Next
                    </Button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-center py-12 text-muted-foreground">
                <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
                <p>No results found</p>
                <p className="text-sm">Try adjusting your search terms or filters</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Search Error */}
      {searchMutation.error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <div className="text-center text-destructive">
              <p>Search failed</p>
              <p className="text-sm mt-2">Please try again or adjust your search terms</p>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Search Tips */}
      {!searchMutation.data && !searchMutation.isLoading && (
        <Card>
          <CardHeader>
            <CardTitle>Search Tips</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <h4 className="font-medium mb-2">Basic Search</h4>
                <ul className="space-y-1 text-muted-foreground">
                  <li>• Use keywords to find relevant content</li>
                  <li>• Search is case-insensitive</li>
                  <li>• Multiple words are treated as AND</li>
                </ul>
              </div>
              <div>
                <h4 className="font-medium mb-2">Advanced Search</h4>
                <ul className="space-y-1 text-muted-foreground">
                  <li>• Use quotes for exact phrases: "machine learning"</li>
                  <li>• Use filters to narrow results by type and date</li>
                  <li>• Sort results by relevance or date</li>
                </ul>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};