/**
 * Production Status Dashboard
 *
 * Real-time monitoring of recording status, performance metrics,
 * and system health for production deployments.
 */

import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Cpu,
  HardDrive,
  Radio,
  Clock
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface RecordingMetrics {
  fps: number;
  frame_drops: number;
  bitrate_kbps: number;
  cpu_percent: number;
  memory_mb: number;
  buffer_segments: number;
  buffer_size_mb: number;
}

interface SystemMetrics {
  total_cpu_percent: number;
  available_ram_gb: number;
  available_disk_gb: number;
  gpu_percent?: number;
  gpu_memory_mb?: number;
}

type HealthStatus = 'Healthy' | 'Warning' | 'Critical';

interface RecordingStatus {
  status: 'Idle' | 'Buffering' | 'Recording' | 'Error';
  is_monitoring: boolean;
  buffer_duration_secs: number;
}

export function StatusDashboard() {
  const [recordingMetrics, setRecordingMetrics] = useState<RecordingMetrics | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [healthStatus, setHealthStatus] = useState<HealthStatus>('Healthy');
  const [recordingStatus, setRecordingStatus] = useState<RecordingStatus>({
    status: 'Idle',
    is_monitoring: false,
    buffer_duration_secs: 0
  });

  useEffect(() => {
    // Poll metrics every 2 seconds
    const interval = setInterval(async () => {
      try {
        // Fetch recording status
        const status = await invoke<RecordingStatus>('get_recording_status');
        setRecordingStatus(status);

        // Fetch performance metrics if recording
        if (status.status === 'Buffering' || status.status === 'Recording') {
          // Note: These would be actual backend commands
          // For now, simulating with placeholder data
          const recMetrics: RecordingMetrics = {
            fps: 60.0,
            frame_drops: 0,
            bitrate_kbps: 8000,
            cpu_percent: 45.0,
            memory_mb: 512.0,
            buffer_segments: 6,
            buffer_size_mb: 1250.0
          };
          setRecordingMetrics(recMetrics);

          const sysMetrics: SystemMetrics = {
            total_cpu_percent: 35.0,
            available_ram_gb: 8.5,
            available_disk_gb: 150.0
          };
          setSystemMetrics(sysMetrics);

          // Determine health status
          if (recMetrics.fps < 45 || recMetrics.cpu_percent > 95) {
            setHealthStatus('Critical');
          } else if (recMetrics.fps < 55 || recMetrics.cpu_percent > 80) {
            setHealthStatus('Warning');
          } else {
            setHealthStatus('Healthy');
          }
        }
      } catch (error) {
        console.error('Failed to fetch metrics:', error);
      }
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  const getStatusBadgeVariant = (status: string) => {
    switch (status) {
      case 'Buffering': return 'default';
      case 'Recording': return 'destructive';
      case 'Error': return 'destructive';
      default: return 'secondary';
    }
  };

  const getHealthBadgeVariant = (health: HealthStatus) => {
    switch (health) {
      case 'Healthy': return 'default';
      case 'Warning': return 'secondary';
      case 'Critical': return 'destructive';
    }
  };

  const getHealthIcon = (health: HealthStatus) => {
    switch (health) {
      case 'Healthy': return <CheckCircle2 className="h-4 w-4" />;
      case 'Warning': return <AlertTriangle className="h-4 w-4" />;
      case 'Critical': return <XCircle className="h-4 w-4" />;
    }
  };

  return (
    <div className="space-y-4">
      {/* Recording Status Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Radio className="h-5 w-5" />
              Recording Status
            </CardTitle>
            <Badge variant={getStatusBadgeVariant(recordingStatus.status)}>
              {recordingStatus.status}
            </Badge>
          </div>
          <CardDescription>
            {recordingStatus.is_monitoring
              ? 'Auto-capture active - monitoring game events'
              : 'Press F8 to start auto-capture'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <div className="text-sm text-muted-foreground">Buffer Duration</div>
              <div className="text-2xl font-bold">{recordingStatus.buffer_duration_secs}s</div>
            </div>
            {recordingMetrics && (
              <div>
                <div className="text-sm text-muted-foreground">Buffer Segments</div>
                <div className="text-2xl font-bold">{recordingMetrics.buffer_segments}/6</div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Performance Metrics */}
      {recordingMetrics && (
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2">
                <Activity className="h-5 w-5" />
                Performance Metrics
              </CardTitle>
              <Badge variant={getHealthBadgeVariant(healthStatus)}>
                {getHealthIcon(healthStatus)}
                <span className="ml-1">{healthStatus}</span>
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* FPS */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">FPS</span>
                <span className="text-sm text-muted-foreground">
                  {recordingMetrics.fps.toFixed(1)} / 60
                </span>
              </div>
              <Progress
                value={(recordingMetrics.fps / 60) * 100}
                className={recordingMetrics.fps < 55 ? 'bg-yellow-500' : ''}
              />
            </div>

            {/* CPU */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium flex items-center gap-1">
                  <Cpu className="h-4 w-4" />
                  FFmpeg CPU
                </span>
                <span className="text-sm text-muted-foreground">
                  {recordingMetrics.cpu_percent.toFixed(1)}%
                </span>
              </div>
              <Progress
                value={recordingMetrics.cpu_percent}
                className={recordingMetrics.cpu_percent > 80 ? 'bg-orange-500' : ''}
              />
            </div>

            {/* Memory */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium flex items-center gap-1">
                  <HardDrive className="h-4 w-4" />
                  Memory
                </span>
                <span className="text-sm text-muted-foreground">
                  {recordingMetrics.memory_mb.toFixed(0)} MB
                </span>
              </div>
              <Progress value={(recordingMetrics.memory_mb / 2048) * 100} />
            </div>

            {/* Bitrate */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">Bitrate</span>
                <span className="text-sm text-muted-foreground">
                  {(recordingMetrics.bitrate_kbps / 1000).toFixed(1)} Mbps
                </span>
              </div>
            </div>

            {/* Frame Drops */}
            {recordingMetrics.frame_drops > 0 && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Frame Drops Detected</AlertTitle>
                <AlertDescription>
                  {recordingMetrics.frame_drops} frames dropped in current segment
                </AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      )}

      {/* System Resources */}
      {systemMetrics && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Cpu className="h-5 w-5" />
              System Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-sm text-muted-foreground">Total CPU</div>
                <div className="text-2xl font-bold">
                  {systemMetrics.total_cpu_percent.toFixed(1)}%
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">Available RAM</div>
                <div className="text-2xl font-bold">
                  {systemMetrics.available_ram_gb.toFixed(1)} GB
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">Available Disk</div>
                <div className="text-2xl font-bold">
                  {systemMetrics.available_disk_gb.toFixed(1)} GB
                </div>
              </div>
              {systemMetrics.gpu_percent && (
                <div>
                  <div className="text-sm text-muted-foreground">GPU</div>
                  <div className="text-2xl font-bold">
                    {systemMetrics.gpu_percent.toFixed(1)}%
                  </div>
                </div>
              )}
            </div>

            {/* Low disk warning */}
            {systemMetrics.available_disk_gb < 5 && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Low Disk Space</AlertTitle>
                <AlertDescription>
                  Less than 5 GB available. Recording may fail.
                </AlertDescription>
              </Alert>
            )}
          </CardContent>
        </Card>
      )}

      {/* Hotkey Reference */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Hotkey Reference
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Toggle Auto-Capture:</span>
              <kbd className="px-2 py-1 bg-muted rounded">F8</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Save 60s Replay:</span>
              <kbd className="px-2 py-1 bg-muted rounded">F9</kbd>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Save 30s Replay:</span>
              <kbd className="px-2 py-1 bg-muted rounded">F10</kbd>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
