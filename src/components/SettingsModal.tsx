import { useEffect, useState } from "react";
import {
  FolderOpen,
  Gauge,
  Loader2,
  Monitor,
  Moon,
  Palette,
  RefreshCw,
  Settings2,
  Sparkles,
  Sun,
} from "lucide-react";
import { cn } from "../lib/utils.ts";
import { getConfig, updateConfig } from "../commands.ts";
import type { AppConfig } from "../types.ts";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onOpenUpdate?: () => void;
}

type Theme = "light" | "dark" | "system";

export function SettingsModal(
  { isOpen, onClose, onOpenUpdate }: SettingsModalProps,
) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  const [theme, setTheme] = useState<Theme>("system");
  const [defaultPath, setDefaultPath] = useState("");
  const [maxTasks, setMaxTasks] = useState("3");
  const [maxDownloads, setMaxDownloads] = useState("4");

  // Load config when modal opens
  useEffect(() => {
    if (isOpen && !config) {
      setLoading(true);
      getConfig()
        .then((cfg) => {
          setConfig(cfg);
          setTheme(cfg.theme as Theme);
          setDefaultPath(cfg.default_download_path);
          setMaxTasks(cfg.max_concurrent_tasks.toString());
          setMaxDownloads(cfg.max_concurrent_downloads.toString());
        })
        .catch((err) => {
          console.error("Failed to load config:", err);
          alert("加载配置失败: " + err);
        })
        .finally(() => {
          setLoading(false);
        });
    }
  }, [isOpen, config]);

  // Apply theme changes
  useEffect(() => {
    const root = document.documentElement;
    if (theme === "system") {
      root.removeAttribute("data-theme");
      const systemTheme =
        globalThis.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dim"
          : "cupcake";
      root.setAttribute("data-theme", systemTheme);
    } else {
      root.setAttribute("data-theme", theme === "dark" ? "dim" : "cupcake");
    }
  }, [theme]);

  const handleSave = async () => {
    if (!config) return;

    setSaving(true);
    try {
      const newConfig: AppConfig = {
        ...config,
        theme,
        default_download_path: defaultPath,
        max_concurrent_tasks: parseInt(maxTasks),
        max_concurrent_downloads: parseInt(maxDownloads),
      };

      await updateConfig(newConfig);
      setConfig(newConfig);
      onClose();
    } catch (err) {
      console.error("Failed to save config:", err);
      alert("保存配置失败: " + err);
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center animate-in fade-in">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/30 backdrop-blur-md"
        onClick={onClose}
      />

      {/* Settings Window */}
      <div className="relative w-[680px] max-w-[90vw] h-[520px] max-h-[85vh] bg-base-100 rounded-2xl shadow-2xl overflow-hidden flex animate-in zoom-in-95 duration-200 border border-base-300/50">
        {/* Sidebar */}
        <div className="w-[200px] bg-linear-to-b from-base-200/80 to-base-200/50 border-r border-base-300/50 p-4 pt-12">
          <div className="space-y-2">
            <button
              type="button"
              className="w-full text-left px-4 py-2.5 rounded-xl text-sm font-semibold bg-primary text-primary-content shadow-lg shadow-primary/20 flex items-center gap-2"
            >
              <Settings2 className="w-4 h-4" />
              通用
            </button>
            <button
              type="button"
              className="w-full text-left px-4 py-2.5 rounded-xl text-sm text-base-content/60 hover:text-base-content hover:bg-base-100/50 transition-all flex items-center gap-2"
            >
              <Gauge className="w-4 h-4" />
              网络
            </button>
            <button
              type="button"
              className="w-full text-left px-4 py-2.5 rounded-xl text-sm text-base-content/60 hover:text-base-content hover:bg-base-100/50 transition-all flex items-center gap-2"
            >
              <Sparkles className="w-4 h-4" />
              高级
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 flex flex-col min-w-0 bg-base-100">
          {/* Header */}
          <div className="h-12 border-b border-base-200 flex items-center justify-center bg-linear-to-r from-primary/5 via-transparent to-secondary/5 relative draggable">
            <span className="text-sm font-bold text-base-content flex items-center gap-2">
              <Settings2 className="w-4 h-4 text-primary" />
              通用设置
            </span>
          </div>

          {/* Scrollable Content */}
          <div className="p-8 space-y-8 overflow-y-auto flex-1">
            {loading
              ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 className="w-8 h-8 animate-spin text-primary" />
                </div>
              )
              : (
                <>
                  {/* Theme Section */}
                  <div className="space-y-4">
                    <label className="text-sm font-bold text-base-content flex items-center gap-2">
                      <Palette className="w-4 h-4 text-primary" />
                      外观主题
                    </label>
                    <div className="grid grid-cols-3 gap-3">
                      <button
                        type="button"
                        onClick={() => setTheme("light")}
                        className={cn(
                          "flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all hover:scale-105",
                          theme === "light"
                            ? "bg-linear-to-br from-primary/10 to-secondary/10 border-primary shadow-lg shadow-primary/20"
                            : "bg-base-100 border-base-300 hover:border-base-content/20",
                        )}
                      >
                        <div
                          className={cn(
                            "p-2.5 rounded-lg",
                            theme === "light" ? "bg-primary/20" : "bg-base-200",
                          )}
                        >
                          <Sun
                            className={cn(
                              "w-6 h-6",
                              theme === "light"
                                ? "text-primary"
                                : "text-base-content/60",
                            )}
                          />
                        </div>
                        <span
                          className={cn(
                            "text-sm font-medium",
                            theme === "light"
                              ? "text-primary"
                              : "text-base-content/70",
                          )}
                        >
                          浅色
                        </span>
                      </button>
                      <button
                        type="button"
                        onClick={() => setTheme("dark")}
                        className={cn(
                          "flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all hover:scale-105",
                          theme === "dark"
                            ? "bg-linear-to-br from-primary/10 to-secondary/10 border-primary shadow-lg shadow-primary/20"
                            : "bg-base-100 border-base-300 hover:border-base-content/20",
                        )}
                      >
                        <div
                          className={cn(
                            "p-2.5 rounded-lg",
                            theme === "dark" ? "bg-primary/20" : "bg-base-200",
                          )}
                        >
                          <Moon
                            className={cn(
                              "w-6 h-6",
                              theme === "dark"
                                ? "text-primary"
                                : "text-base-content/60",
                            )}
                          />
                        </div>
                        <span
                          className={cn(
                            "text-sm font-medium",
                            theme === "dark"
                              ? "text-primary"
                              : "text-base-content/70",
                          )}
                        >
                          深色
                        </span>
                      </button>
                      <button
                        type="button"
                        onClick={() => setTheme("system")}
                        className={cn(
                          "flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all hover:scale-105",
                          theme === "system"
                            ? "bg-linear-to-br from-primary/10 to-secondary/10 border-primary shadow-lg shadow-primary/20"
                            : "bg-base-100 border-base-300 hover:border-base-content/20",
                        )}
                      >
                        <div
                          className={cn(
                            "p-2.5 rounded-lg",
                            theme === "system"
                              ? "bg-primary/20"
                              : "bg-base-200",
                          )}
                        >
                          <Monitor
                            className={cn(
                              "w-6 h-6",
                              theme === "system"
                                ? "text-primary"
                                : "text-base-content/60",
                            )}
                          />
                        </div>
                        <span
                          className={cn(
                            "text-sm font-medium",
                            theme === "system"
                              ? "text-primary"
                              : "text-base-content/70",
                          )}
                        >
                          自动
                        </span>
                      </button>
                    </div>
                    <p className="text-xs text-base-content/50 pl-6">
                      跟随系统主题自动切换浅色和深色模式
                    </p>
                  </div>

                  <div className="divider my-2"></div>

                  {/* Download Path */}
                  <div className="space-y-3">
                    <label className="text-sm font-bold text-base-content flex items-center gap-2">
                      <FolderOpen className="w-4 h-4 text-primary" />
                      默认下载路径
                    </label>
                    <div className="join w-full shadow-sm">
                      <div className="join-item flex-1 px-4 py-3 bg-base-200 border border-base-300 rounded-l-xl text-sm text-base-content/70 font-mono truncate flex items-center">
                        {defaultPath}
                      </div>
                      <button
                        type="button"
                        className="btn join-item btn-outline rounded-r-xl h-auto min-h-0 font-medium hover:btn-primary transition-all"
                      >
                        <FolderOpen className="w-4 h-4" />
                        更改
                      </button>
                    </div>
                    <p className="text-xs text-base-content/50 pl-6">
                      新建任务时的默认保存位置
                    </p>
                  </div>

                  {/* Max Tasks */}
                  <div className="space-y-4">
                    <div className="flex justify-between items-center">
                      <label className="text-sm font-bold text-base-content flex items-center gap-2">
                        <Gauge className="w-4 h-4 text-primary" />
                        最大同时下载数
                      </label>
                      <div className="badge badge-primary badge-lg font-mono font-bold">
                        {maxTasks}
                      </div>
                    </div>
                    <input
                      type="range"
                      min="1"
                      max="10"
                      step="1"
                      value={maxTasks}
                      onChange={(e) => setMaxTasks(e.target.value)}
                      className="range range-primary"
                    />
                    <div className="flex justify-between text-xs text-base-content/50 font-medium px-1">
                      <span>1</span>
                      <span>3</span>
                      <span>5</span>
                      <span>7</span>
                      <span>10</span>
                    </div>
                    <p className="text-xs text-base-content/50 pl-6">
                      控制同时进行的下载任务数量
                    </p>
                  </div>

                  <div className="divider my-2"></div>

                  {/* Max Downloads per Task */}
                  <div className="space-y-4">
                    <div className="flex justify-between items-center">
                      <label className="text-sm font-bold text-base-content flex items-center gap-2">
                        <Gauge className="w-4 h-4 text-primary" />
                        每个任务的最大连接数
                      </label>
                      <div className="badge badge-secondary badge-lg font-mono font-bold">
                        {maxDownloads}
                      </div>
                    </div>
                    <input
                      type="range"
                      min="1"
                      max="16"
                      step="1"
                      value={maxDownloads}
                      onChange={(e) => setMaxDownloads(e.target.value)}
                      className="range range-secondary"
                    />
                    <div className="flex justify-between text-xs text-base-content/50 font-medium px-1">
                      <span>1</span>
                      <span>4</span>
                      <span>8</span>
                      <span>12</span>
                      <span>16</span>
                    </div>
                    <p className="text-xs text-base-content/50 pl-6">
                      控制单个任务的并发下载连接数（分块下载）
                    </p>
                  </div>

                  <div className="divider my-2"></div>

                  {/* Check for Updates */}
                  <div className="space-y-3">
                    <label className="text-sm font-bold text-base-content flex items-center gap-2">
                      <RefreshCw className="w-4 h-4 text-primary" />
                      应用更新
                    </label>
                    <button
                      type="button"
                      onClick={() => {
                        onClose();
                        onOpenUpdate?.();
                      }}
                      className="btn btn-outline w-full justify-start gap-3 hover:btn-primary transition-all"
                    >
                      <Sparkles className="w-5 h-5" />
                      <div className="flex-1 text-left">
                        <div className="font-semibold">检查更新</div>
                        <div className="text-xs opacity-70">
                          查看是否有新版本可用
                        </div>
                      </div>
                    </button>
                    <p className="text-xs text-base-content/50 pl-6">
                      当前版本: v0.1.0
                    </p>
                  </div>
                </>
              )}
          </div>

          {/* Footer Actions */}
          <div className="px-8 py-4 border-t border-base-200 bg-linear-to-r from-base-200/30 via-transparent to-base-200/30 flex justify-end gap-3">
            <button
              type="button"
              onClick={onClose}
              disabled={saving}
              className="btn btn-ghost h-10 min-h-0 font-medium hover:bg-base-200 transition-all"
            >
              取消
            </button>
            <button
              type="button"
              onClick={handleSave}
              disabled={saving || loading}
              className="btn btn-primary h-10 min-h-0 font-semibold text-primary-content shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/35 hover:scale-105 transition-all flex items-center gap-2"
            >
              {saving && <Loader2 className="w-4 h-4 animate-spin" />}
              {saving ? "保存中..." : "保存更改"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
