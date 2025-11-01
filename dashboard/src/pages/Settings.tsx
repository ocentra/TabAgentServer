import React from 'react';
import { PageHeader } from '@/components/layout/PageHeader';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import ThemeToggle from '@/components/ui/ThemeToggle';

const Settings: React.FC = () => {
  return (
    <div>
      <PageHeader
        title="Settings"
        description="Configure your TabAgent server and dashboard preferences"
        actions={
          <Button>Save Changes</Button>
        }
      />

      <div className="space-y-6">
        {/* Appearance Settings */}
        <Card>
          <CardHeader>
            <CardTitle>Appearance</CardTitle>
            <CardDescription>Customize the look and feel of your dashboard</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Theme</label>
                <p className="text-sm text-muted-foreground">Choose your preferred color scheme</p>
              </div>
              <ThemeToggle variant="dropdown" />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Compact Mode</label>
                <p className="text-sm text-muted-foreground">Reduce spacing for more content</p>
              </div>
              <input type="checkbox" className="rounded" />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Animations</label>
                <p className="text-sm text-muted-foreground">Enable smooth transitions and effects</p>
              </div>
              <input type="checkbox" className="rounded" defaultChecked />
            </div>
          </CardContent>
        </Card>

        {/* Server Configuration */}
        <Card>
          <CardHeader>
            <CardTitle>Server Configuration</CardTitle>
            <CardDescription>Configure TabAgent server settings and endpoints</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Input label="Server Host" defaultValue="localhost" />
              <Input label="Server Port" defaultValue="3000" type="number" />
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Input label="WebSocket Port" defaultValue="3001" type="number" />
              <Input label="Max Connections" defaultValue="100" type="number" />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Enable CORS</label>
                <p className="text-sm text-muted-foreground">Allow cross-origin requests</p>
              </div>
              <input type="checkbox" className="rounded" defaultChecked />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Enable Logging</label>
                <p className="text-sm text-muted-foreground">Log all server requests and responses</p>
              </div>
              <input type="checkbox" className="rounded" defaultChecked />
            </div>
          </CardContent>
        </Card>

        {/* Model Settings */}
        <Card>
          <CardHeader>
            <CardTitle>Model Configuration</CardTitle>
            <CardDescription>Default settings for model loading and inference</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium mb-2 block">Default Model Path</label>
                <Input placeholder="/path/to/models" />
              </div>
              <div>
                <label className="text-sm font-medium mb-2 block">Max Model Memory</label>
                <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                  <option>4 GB</option>
                  <option>8 GB</option>
                  <option>16 GB</option>
                  <option>32 GB</option>
                </select>
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium mb-2 block">Default Quantization</label>
                <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                  <option>4-bit</option>
                  <option>8-bit</option>
                  <option>16-bit</option>
                  <option>32-bit</option>
                </select>
              </div>
              <div>
                <label className="text-sm font-medium mb-2 block">Context Length</label>
                <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                  <option>2048</option>
                  <option>4096</option>
                  <option>8192</option>
                  <option>16384</option>
                </select>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Auto-load Models</label>
                <p className="text-sm text-muted-foreground">Automatically load models on server start</p>
              </div>
              <input type="checkbox" className="rounded" />
            </div>
          </CardContent>
        </Card>

        {/* Database Settings */}
        <Card>
          <CardHeader>
            <CardTitle>Database Configuration</CardTitle>
            <CardDescription>Configure database connection and storage settings</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Input label="Database URL" defaultValue="sqlite:///data/tabagent.db" />
              <Input label="Connection Pool Size" defaultValue="10" type="number" />
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium mb-2 block">Backup Frequency</label>
                <select className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm">
                  <option>Never</option>
                  <option>Daily</option>
                  <option>Weekly</option>
                  <option>Monthly</option>
                </select>
              </div>
              <Input label="Max Database Size" defaultValue="10 GB" />
            </div>
            <div className="flex items-center justify-between">
              <div>
                <label className="text-sm font-medium">Enable Indexing</label>
                <p className="text-sm text-muted-foreground">Automatically index new documents</p>
              </div>
              <input type="checkbox" className="rounded" defaultChecked />
            </div>
          </CardContent>
        </Card>

        {/* Actions */}
        <div className="flex justify-end space-x-2">
          <Button variant="outline">Reset to Defaults</Button>
          <Button>Save All Settings</Button>
        </div>

        <div className="text-center">
          <p className="text-sm text-muted-foreground">
            Interactive settings management will be implemented in later tasks
          </p>
        </div>
      </div>
    </div>
  );
};

export default Settings;