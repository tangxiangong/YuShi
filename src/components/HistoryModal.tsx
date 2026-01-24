import { useEffect, useState } from "react";
import {
  Clock,
  FileIcon,
  Gauge,
  History,
  Search,
  Trash2,
  X,
} from "lucide-react";
import {
  clearHistory,
  getHistory,
  removeFromHistory,
  searchHistory,
} from "../commands.ts";
import type { CompletedTask } from "../types.ts";
import { formatBytes, formatDuration } from "../utils/format.ts";

interface HistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function HistoryModal({ isOpen, onClose }: HistoryModalProps) {
  const [history, setHistory] = useState<CompletedTask[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(false);

  // Load history when modal opens
  useEffect(() => {
    if (isOpen) {
      loadHistory();
    }
  }, [isOpen]);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await getHistory();
      setHistory(data);
    } catch (err) {
      console.error("Failed to load history:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      loadHistory();
      return;
    }

    setLoading(true);
    try {
      const results = await searchHistory(searchQuery);
      setHistory(results);
    } catch (err) {
      console.error("Failed to search history:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleRemove = async (id: string) => {
    try {
      await removeFromHistory(id);
      setHistory(history.filter((item) => item.id !== id));
    } catch (err) {
      console.error("Failed to remove history item:", err);
      alert("删除失败: " + err);
    }
  };

  const handleClearAll = async () => {
    if (!confirm("确定要清除所有历史记录吗？")) {
      return;
    }

    try {
      await clearHistory();
      setHistory([]);
    } catch (err) {
      console.error("Failed to clear history:", err);
      alert("清除失败: " + err);
    }
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center animate-in fade-in">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/30 backdrop-blur-md"
        onClick={onClose}
      />

      {/* Modal Content */}
      <div className="relative w-[900px] max-w-[90vw] h-[700px] max-h-[85vh] bg-base-100 rounded-2xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-200 border border-base-300/50">
        {/* Header */}
        <div className="px-6 py-4 border-b border-base-200 bg-linear-to-r from-primary/5 via-transparent to-secondary/5 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-lg">
              <History className="w-5 h-5 text-primary" />
            </div>
            <h2 className="text-lg font-bold text-base-content">下载历史</h2>
            <span className="badge badge-neutral">{history.length} 条记录</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="btn btn-sm btn-ghost btn-square"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Search Bar */}
        <div className="px-6 py-4 border-b border-base-200 bg-base-200/30">
          <div className="flex gap-3">
            <div className="join flex-1">
              <input
                type="text"
                placeholder="搜索 URL 或文件路径..."
                className="input input-bordered join-item flex-1"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && handleSearch()}
              />
              <button
                type="button"
                onClick={handleSearch}
                className="btn btn-primary join-item"
              >
                <Search className="w-4 h-4" />
                搜索
              </button>
            </div>
            <button
              type="button"
              onClick={handleClearAll}
              className="btn btn-error btn-outline"
              disabled={history.length === 0}
            >
              <Trash2 className="w-4 h-4" />
              清空
            </button>
          </div>
        </div>

        {/* History List */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading
            ? (
              <div className="flex items-center justify-center py-12">
                <span className="loading loading-spinner loading-lg text-primary">
                </span>
              </div>
            )
            : history.length === 0
            ? (
              <div className="flex flex-col items-center justify-center py-12 text-base-content/50">
                <History className="w-16 h-16 mb-4 opacity-20" />
                <p className="text-sm font-medium">暂无历史记录</p>
              </div>
            )
            : (
              <div className="space-y-3">
                {history.map((item) => (
                  <div
                    key={item.id}
                    className="card bg-base-200 border border-base-300 hover:shadow-md transition-all"
                  >
                    <div className="card-body p-4">
                      <div className="flex items-start gap-3">
                        <div className="p-2 bg-success/10 rounded-lg">
                          <FileIcon className="w-5 h-5 text-success" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <h3
                            className="font-semibold text-sm text-base-content truncate mb-1"
                            title={item.url}
                          >
                            {item.url.split("/").pop() || item.url}
                          </h3>
                          <p
                            className="text-xs text-base-content/60 truncate mb-2"
                            title={item.dest}
                          >
                            {item.dest}
                          </p>
                          <div className="flex items-center gap-4 text-xs text-base-content/70">
                            <span className="flex items-center gap-1">
                              <FileIcon className="w-3 h-3" />
                              {formatBytes(item.total_size)}
                            </span>
                            <span className="flex items-center gap-1">
                              <Gauge className="w-3 h-3" />
                              {formatBytes(item.avg_speed)}/s
                            </span>
                            <span className="flex items-center gap-1">
                              <Clock className="w-3 h-3" />
                              {formatDuration(item.duration)}
                            </span>
                            <span className="opacity-60">
                              {formatDate(item.completed_at)}
                            </span>
                          </div>
                        </div>
                        <button
                          type="button"
                          onClick={() => handleRemove(item.id)}
                          className="btn btn-sm btn-ghost btn-square text-error hover:bg-error/10"
                          title="删除"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
        </div>
      </div>
    </div>
  );
}
