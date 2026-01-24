import { DownloadTask } from "../types.ts";
import { formatBytes, formatDuration } from "../utils/format.ts";
import {
  AlertCircle,
  CheckCircle2,
  Clock,
  Download,
  FileIcon,
  Pause,
  Play,
  RefreshCw,
  Trash2,
  X,
} from "lucide-react";
import { cancelTask, pauseTask, removeTask, resumeTask } from "../commands.ts";
import { cn } from "../lib/utils.ts";

interface TaskItemProps {
  task: DownloadTask;
  onRefreshNeeded: () => void;
}

export function TaskItem({ task, onRefreshNeeded }: TaskItemProps) {
  const progress = task.total_size > 0
    ? (task.downloaded / task.total_size) * 100
    : 0;

  const handlePause = async () => {
    await pauseTask(task.id);
    onRefreshNeeded();
  };
  const handleResume = async () => {
    await resumeTask(task.id);
    onRefreshNeeded();
  };
  const handleCancel = async () => {
    await cancelTask(task.id);
    onRefreshNeeded();
  };
  const handleRemove = async () => {
    await removeTask(task.id);
    onRefreshNeeded();
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "Completed":
        return (
          <div className="badge badge-success badge-sm gap-1.5 font-medium shadow-sm">
            <CheckCircle2 className="w-3 h-3" />
            已完成
          </div>
        );
      case "Downloading":
        return (
          <div className="badge badge-primary badge-sm gap-1.5 font-medium shadow-sm animate-pulse">
            <Download className="w-3 h-3" />
            下载中
          </div>
        );
      case "Paused":
        return (
          <div className="badge badge-warning badge-sm gap-1.5 font-medium shadow-sm">
            <Pause className="w-3 h-3" />
            已暂停
          </div>
        );
      case "Failed":
        return (
          <div className="badge badge-error badge-sm gap-1.5 font-medium shadow-sm">
            <AlertCircle className="w-3 h-3" />
            失败
          </div>
        );
      case "Cancelled":
        return (
          <div className="badge badge-neutral badge-sm gap-1.5 font-medium shadow-sm">
            <X className="w-3 h-3" />
            已取消
          </div>
        );
      default:
        return (
          <div className="badge badge-ghost badge-sm gap-1.5 font-medium shadow-sm">
            <Clock className="w-3 h-3" />
            等待中
          </div>
        );
    }
  };

  const getProgressColor = (status: string) => {
    switch (status) {
      case "Completed":
        return "progress-success";
      case "Failed":
        return "progress-error";
      case "Paused":
        return "progress-warning";
      default:
        return "progress-primary";
    }
  };

  const getCardStyle = (status: string) => {
    switch (status) {
      case "Completed":
        return "border-success/20 bg-success/5";
      case "Downloading":
        return "border-primary/30 bg-primary/5 shadow-lg shadow-primary/10";
      case "Failed":
        return "border-error/20 bg-error/5";
      default:
        return "border-base-300 bg-base-100";
    }
  };

  return (
    <div
      className={cn(
        "card shadow-md border hover:shadow-xl transition-all duration-300 group",
        getCardStyle(task.status),
      )}
    >
      <div className="card-body p-5 sm:p-6">
        <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center">
          {/* Icon & File Info */}
          <div className="flex-1 min-w-0 w-full">
            <div className="flex items-center gap-3 mb-3">
              <div className="relative">
                <div
                  className={cn(
                    "absolute inset-0 rounded-xl blur-md opacity-30",
                    task.status === "Downloading" && "bg-primary animate-pulse",
                    task.status === "Completed" && "bg-success",
                    task.status === "Failed" && "bg-error",
                  )}
                >
                </div>
                <div
                  className={cn(
                    "relative p-2.5 rounded-xl shadow-sm transition-all duration-200 group-hover:scale-105",
                    task.status === "Downloading" && "bg-primary/10",
                    task.status === "Completed" && "bg-success/10",
                    task.status === "Failed" && "bg-error/10",
                    task.status === "Paused" && "bg-warning/10",
                    !["Downloading", "Completed", "Failed", "Paused"].includes(
                      task.status,
                    ) && "bg-base-300",
                  )}
                >
                  <FileIcon
                    className={cn(
                      "w-5 h-5",
                      task.status === "Downloading" && "text-primary",
                      task.status === "Completed" && "text-success",
                      task.status === "Failed" && "text-error",
                      task.status === "Paused" && "text-warning",
                      !["Downloading", "Completed", "Failed", "Paused"]
                        .includes(task.status) && "text-base-content/70",
                    )}
                  />
                </div>
              </div>
              <div className="flex-1 min-w-0">
                <h3
                  className="font-semibold text-base text-base-content truncate mb-1"
                  title={task.url}
                >
                  {task.url.split("/").pop() || task.url}
                </h3>
                <div className="flex items-center gap-2 text-xs text-base-content/60 flex-wrap">
                  {getStatusBadge(task.status)}
                  <span className="hidden sm:inline">•</span>
                  <span
                    className="truncate font-mono opacity-70 max-w-xs"
                    title={task.dest}
                  >
                    {task.dest.split("/").slice(-2).join("/")}
                  </span>
                </div>
              </div>
            </div>

            {/* Progress & Stats */}
            <div className="space-y-2.5">
              <div className="relative">
                <progress
                  className={cn(
                    "progress w-full h-2.5 shadow-inner",
                    getProgressColor(task.status),
                  )}
                  value={progress}
                  max="100"
                >
                </progress>
                {task.status === "Downloading" && (
                  <div className="absolute inset-0 bg-linear-to-r from-transparent via-white/20 to-transparent animate-pulse">
                  </div>
                )}
              </div>

              <div className="flex justify-between items-center text-xs font-mono text-base-content/70">
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-base-content">
                    {progress.toFixed(1)}%
                  </span>
                  <span className="opacity-60">•</span>
                  <span>
                    {formatBytes(task.downloaded)} /{" "}
                    {formatBytes(task.total_size)}
                  </span>
                </div>
                {task.status === "Downloading" && (
                  <div className="flex gap-3 items-center">
                    <span className="text-primary font-semibold">
                      {formatBytes(task.speed)}/s
                    </span>
                    <span className="opacity-60">•</span>
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatDuration(task.eta || 0)}
                    </span>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex sm:flex-col gap-2 self-end sm:self-center">
            {task.status === "Downloading" && (
              <button
                type="button"
                onClick={handlePause}
                className="btn btn-sm btn-ghost btn-square hover:bg-warning/10 hover:text-warning transition-all duration-200"
                title="暂停"
              >
                <Pause className="w-4 h-4" />
              </button>
            )}
            {task.status === "Paused" && (
              <button
                type="button"
                onClick={handleResume}
                className="btn btn-sm btn-ghost btn-square hover:bg-primary/10 hover:text-primary transition-all duration-200"
                title="继续"
              >
                <Play className="w-4 h-4" />
              </button>
            )}
            {task.status === "Failed" && (
              <button
                type="button"
                onClick={handleResume}
                className="btn btn-sm btn-warning hover:btn-primary transition-all duration-200 gap-1"
                title="重试下载"
              >
                <RefreshCw className="w-4 h-4" />
                <span className="hidden sm:inline">重试</span>
              </button>
            )}
            {(task.status === "Downloading" || task.status === "Paused" ||
              task.status === "Pending") && (
              <button
                type="button"
                onClick={handleCancel}
                className="btn btn-sm btn-ghost btn-square text-error hover:bg-error/10 transition-all duration-200"
                title="取消"
              >
                <X className="w-4 h-4" />
              </button>
            )}
            {(task.status === "Completed" || task.status === "Cancelled" ||
              task.status === "Failed") && (
              <button
                type="button"
                onClick={handleRemove}
                className="btn btn-sm btn-ghost btn-square text-error hover:bg-error/10 transition-all duration-200"
                title="移除"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
