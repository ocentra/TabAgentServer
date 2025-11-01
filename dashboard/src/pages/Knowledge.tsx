import React, { useState, useRef } from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

interface EmbeddingJob {
  id: string;
  name: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  progress: number;
  totalFiles: number;
  processedFiles: number;
  startTime: Date;
  endTime?: Date;
  model: 'qwen-0.6b' | 'qwen-8b';
}

interface Document {
  id: string;
  name: string;
  type: 'pdf' | 'docx' | 'txt' | 'image' | 'video';
  size: number;
  chunks: number;
  embeddings: {
    lowRes: boolean;
    highRes: boolean;
  };
  uploadDate: Date;
}

const Knowledge: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'upload' | 'jobs' | 'documents' | 'search' | 'evaluation'>('upload');
  const [uploadProgress, setUploadProgress] = useState(0);
  const [isUploading, setIsUploading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const folderInputRef = useRef<HTMLInputElement>(null);

  // Mock data - will be replaced with real API calls
  const [embeddingJobs] = useState<EmbeddingJob[]>([
    {
      id: '1',
      name: 'Research Papers Batch',
      status: 'processing',
      progress: 65,
      totalFiles: 24,
      processedFiles: 16,
      startTime: new Date(Date.now() - 1800000),
      model: 'qwen-8b'
    },
    {
      id: '2',
      name: 'Documentation Set',
      status: 'completed',
      progress: 100,
      totalFiles: 12,
      processedFiles: 12,
      startTime: new Date(Date.now() - 3600000),
      endTime: new Date(Date.now() - 1200000),
      model: 'qwen-0.6b'
    }
  ]);

  const [documents] = useState<Document[]>([
    {
      id: '1',
      name: 'AI Research Paper.pdf',
      type: 'pdf',
      size: 2048576,
      chunks: 45,
      embeddings: { lowRes: true, highRes: true },
      uploadDate: new Date(Date.now() - 86400000)
    },
    {
      id: '2',
      name: 'Technical Documentation.docx',
      type: 'docx',
      size: 1024000,
      chunks: 23,
      embeddings: { lowRes: true, highRes: false },
      uploadDate: new Date(Date.now() - 172800000)
    }
  ]);

  const handleFileUpload = async (files: FileList | null) => {
    if (!files) return;
    
    setIsUploading(true);
    setUploadProgress(0);
    
    // Simulate upload progress
    for (let i = 0; i <= 100; i += 10) {
      setUploadProgress(i);
      await new Promise(resolve => setTimeout(resolve, 200));
    }
    
    setIsUploading(false);
    setUploadProgress(0);
  };

  const handleSimilaritySearch = async () => {
    if (!searchQuery.trim()) return;
    
    // Mock search results
    setSearchResults([
      {
        id: '1',
        content: 'Machine learning is a subset of artificial intelligence that focuses on algorithms...',
        similarity: 0.92,
        document: 'AI Research Paper.pdf',
        chunk: 12
      },
      {
        id: '2',
        content: 'Neural networks are computing systems inspired by biological neural networks...',
        similarity: 0.87,
        document: 'Technical Documentation.docx',
        chunk: 8
      }
    ]);
  };

  const formatFileSize = (bytes: number) => {
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 Bytes';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  const getStatusColor = (status: EmbeddingJob['status']) => {
    switch (status) {
      case 'pending': return 'bg-warning-100 text-warning-800 dark:bg-warning-900 dark:text-warning-200';
      case 'processing': return 'bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200';
      case 'completed': return 'bg-success-100 text-success-800 dark:bg-success-900 dark:text-success-200';
      case 'failed': return 'bg-error-100 text-error-800 dark:bg-error-900 dark:text-error-200';
    }
  };

  return (
    <div>
      <PageHeader
        title="Knowledge Base"
        description="Manage document embeddings, RAG pipeline, and knowledge extraction for AI agents"
        actions={
          <div className="flex space-x-2">
            <Button variant="outline">Export Knowledge</Button>
            <Button>New Embedding Job</Button>
          </div>
        }
      />

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-muted p-1 rounded-lg w-fit">
        {[
          { id: 'upload', label: 'Upload & Process' },
          { id: 'jobs', label: 'Embedding Jobs' },
          { id: 'documents', label: 'Documents' },
          { id: 'search', label: 'Similarity Search' },
          { id: 'evaluation', label: 'RAG Evaluation' }
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
              activeTab === tab.id
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Upload & Process Tab */}
      {activeTab === 'upload' && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Upload Files</CardTitle>
                <CardDescription>Upload individual files for embedding processing</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="border-2 border-dashed border-border rounded-lg p-8 text-center">
                  <svg className="mx-auto h-12 w-12 text-muted-foreground mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                  </svg>
                  <p className="text-sm text-muted-foreground mb-4">
                    Drag and drop files here, or click to select
                  </p>
                  <Button onClick={() => fileInputRef.current?.click()}>
                    Select Files
                  </Button>
                  <input
                    ref={fileInputRef}
                    type="file"
                    multiple
                    className="hidden"
                    accept=".pdf,.docx,.txt,.png,.jpg,.jpeg,.mp4,.mov"
                    onChange={(e) => handleFileUpload(e.target.files)}
                  />
                </div>
                <p className="text-xs text-muted-foreground">
                  Supported: PDF, DOCX, TXT, Images (PNG, JPG), Videos (MP4, MOV)
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Upload Folder</CardTitle>
                <CardDescription>Process entire folders with batch embedding</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="border-2 border-dashed border-border rounded-lg p-8 text-center">
                  <svg className="mx-auto h-12 w-12 text-muted-foreground mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2-2z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 5a2 2 0 012-2h4a2 2 0 012 2v6H8V5z" />
                  </svg>
                  <p className="text-sm text-muted-foreground mb-4">
                    Select a folder to process all files
                  </p>
                  <Button onClick={() => folderInputRef.current?.click()}>
                    Select Folder
                  </Button>
                  <input
                    ref={folderInputRef}
                    type="file"
                    {...({ webkitdirectory: "" } as any)}
                    className="hidden"
                    onChange={(e) => handleFileUpload(e.target.files)}
                  />
                </div>
              </CardContent>
            </Card>
          </div>

          {isUploading && (
            <Card>
              <CardContent className="pt-6">
                <div className="flex items-center space-x-4">
                  <LoadingSpinner size="sm" />
                  <div className="flex-1">
                    <div className="flex justify-between text-sm mb-2">
                      <span>Uploading files...</span>
                      <span>{uploadProgress}%</span>
                    </div>
                    <div className="w-full bg-muted rounded-full h-2">
                      <div 
                        className="bg-primary-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${uploadProgress}%` }}
                      />
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Embedding Configuration */}
          <Card>
            <CardHeader>
              <CardTitle>Embedding Configuration</CardTitle>
              <CardDescription>Configure embedding models and processing options</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="text-sm font-medium mb-2 block">Low Resolution Model</label>
                  <select className="w-full p-2 border border-input rounded-md bg-background">
                    <option>Qwen 0.6B (Fast, Lower Quality)</option>
                    <option>MiniLM-L6 (Balanced)</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">High Resolution Model</label>
                  <select className="w-full p-2 border border-input rounded-md bg-background">
                    <option>Qwen 8B (Slow, High Quality)</option>
                    <option>BGE-Large (Enterprise)</option>
                  </select>
                </div>
              </div>
              <div className="flex items-center space-x-4">
                <label className="flex items-center space-x-2">
                  <input type="checkbox" defaultChecked className="rounded" />
                  <span className="text-sm">Generate low-res embeddings</span>
                </label>
                <label className="flex items-center space-x-2">
                  <input type="checkbox" defaultChecked className="rounded" />
                  <span className="text-sm">Generate high-res embeddings</span>
                </label>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Embedding Jobs Tab */}
      {activeTab === 'jobs' && (
        <div className="space-y-4">
          {embeddingJobs.map((job) => (
            <Card key={job.id}>
              <CardContent className="pt-6">
                <div className="flex items-center justify-between mb-4">
                  <div>
                    <h3 className="font-medium">{job.name}</h3>
                    <p className="text-sm text-muted-foreground">
                      {job.processedFiles}/{job.totalFiles} files • {job.model}
                    </p>
                  </div>
                  <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(job.status)}`}>
                    {job.status}
                  </span>
                </div>
                
                {job.status === 'processing' && (
                  <div className="mb-4">
                    <div className="flex justify-between text-sm mb-2">
                      <span>Progress</span>
                      <span>{job.progress}%</span>
                    </div>
                    <div className="w-full bg-muted rounded-full h-2">
                      <div 
                        className="bg-primary-500 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${job.progress}%` }}
                      />
                    </div>
                  </div>
                )}
                
                <div className="flex justify-between text-sm text-muted-foreground">
                  <span>Started: {job.startTime.toLocaleString()}</span>
                  {job.endTime && <span>Completed: {job.endTime.toLocaleString()}</span>}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Documents Tab */}
      {activeTab === 'documents' && (
        <Card>
          <CardHeader>
            <CardTitle>Document Library</CardTitle>
            <CardDescription>Browse and manage processed documents</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {documents.map((doc) => (
                <div key={doc.id} className="flex items-center justify-between p-4 border border-border rounded-lg">
                  <div className="flex items-center space-x-4">
                    <div className="w-10 h-10 bg-muted rounded-lg flex items-center justify-center">
                      {doc.type === 'pdf' && '📄'}
                      {doc.type === 'docx' && '📝'}
                      {doc.type === 'txt' && '📋'}
                      {doc.type === 'image' && '🖼️'}
                      {doc.type === 'video' && '🎥'}
                    </div>
                    <div>
                      <h4 className="font-medium">{doc.name}</h4>
                      <p className="text-sm text-muted-foreground">
                        {formatFileSize(doc.size)} • {doc.chunks} chunks • {doc.uploadDate.toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-4">
                    <div className="text-sm">
                      <span className={`px-2 py-1 rounded-full text-xs ${doc.embeddings.lowRes ? 'bg-success-100 text-success-800' : 'bg-muted text-muted-foreground'}`}>
                        Low-Res
                      </span>
                      <span className={`ml-2 px-2 py-1 rounded-full text-xs ${doc.embeddings.highRes ? 'bg-success-100 text-success-800' : 'bg-muted text-muted-foreground'}`}>
                        High-Res
                      </span>
                    </div>
                    <Button variant="outline" size="sm">View</Button>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Similarity Search Tab */}
      {activeTab === 'search' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Similarity Search</CardTitle>
              <CardDescription>Test embedding similarity and retrieve relevant chunks</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex space-x-2">
                <Input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="Enter your query to find similar content..."
                  className="flex-1"
                />
                <Button onClick={handleSimilaritySearch}>Search</Button>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="text-sm font-medium mb-2 block">Embedding Model</label>
                  <select className="w-full p-2 border border-input rounded-md bg-background">
                    <option>Qwen 0.6B (Fast)</option>
                    <option>Qwen 8B (Accurate)</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">Top K Results</label>
                  <select className="w-full p-2 border border-input rounded-md bg-background">
                    <option>5</option>
                    <option>10</option>
                    <option>20</option>
                  </select>
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">Similarity Threshold</label>
                  <select className="w-full p-2 border border-input rounded-md bg-background">
                    <option>0.7</option>
                    <option>0.8</option>
                    <option>0.9</option>
                  </select>
                </div>
              </div>
            </CardContent>
          </Card>

          {searchResults.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Search Results</CardTitle>
                <CardDescription>Similar content chunks found in your knowledge base</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {searchResults.map((result) => (
                    <div key={result.id} className="p-4 border border-border rounded-lg">
                      <div className="flex justify-between items-start mb-2">
                        <div className="text-sm text-muted-foreground">
                          {result.document} • Chunk {result.chunk}
                        </div>
                        <span className="px-2 py-1 bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200 text-xs rounded-full">
                          {(result.similarity * 100).toFixed(1)}% match
                        </span>
                      </div>
                      <p className="text-sm">{result.content}</p>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* RAG Evaluation Tab */}
      {activeTab === 'evaluation' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle>Evaluation Metrics</CardTitle>
              <CardDescription>RAG system performance and quality metrics</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-success-600">94.2%</div>
                  <div className="text-sm text-muted-foreground">Retrieval Accuracy</div>
                </div>
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-primary-600">87.5%</div>
                  <div className="text-sm text-muted-foreground">Answer Relevance</div>
                </div>
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-warning-600">0.23s</div>
                  <div className="text-sm text-muted-foreground">Avg Response Time</div>
                </div>
                <div className="text-center p-4 bg-muted rounded-lg">
                  <div className="text-2xl font-bold text-secondary-600">92.1%</div>
                  <div className="text-sm text-muted-foreground">Context Precision</div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Test Queries</CardTitle>
              <CardDescription>Run evaluation queries to test RAG performance</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Input placeholder="Enter test query..." />
                <Button className="w-full">Run Evaluation</Button>
              </div>
              
              <div className="space-y-2">
                <h4 className="font-medium text-sm">Quick Tests</h4>
                <div className="space-y-1">
                  <Button variant="outline" size="sm" className="w-full justify-start text-xs">
                    "What is machine learning?"
                  </Button>
                  <Button variant="outline" size="sm" className="w-full justify-start text-xs">
                    "Explain neural networks"
                  </Button>
                  <Button variant="outline" size="sm" className="w-full justify-start text-xs">
                    "How does RAG work?"
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};

export default Knowledge;