import React from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { LogsViewer } from '@/components/features/logs/LogsViewer';

const Logs: React.FC = () => {
  return (
    <div>
      <PageHeader
        title="Real-Time Log Monitor"
        description="Advanced log streaming with real-time updates, filtering, and analysis"
      />
      
      <LogsViewer />
    </div>
  );
};

export default Logs;